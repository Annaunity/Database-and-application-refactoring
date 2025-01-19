use std::net::SocketAddr;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::{ConnectInfo, FromRequestParts, Query, State};
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use axum::{RequestPartsExt, Router};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{TimeDelta, Utc};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Deserialize;
use sha2::{Digest as _, Sha512};

use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{Credentials, Items, Session, Token};

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(auth))
        .route("/", delete(end_current_session))
        .route("/session", get(get_sessions))
        .route("/session", delete(end_session))
}

async fn auth(
    headers: HeaderMap,
    State(globals): State<Globals>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    AppJson(credentials): AppJson<Credentials>,
) -> Result<AppJson<Token>> {
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    let ip_address = addr.ip().to_canonical().to_string();

    let user = sqlx::query!(
        "select username, password_hash from users where username = $1 or email = $1 limit 1",
        credentials.username_or_email
    )
    .fetch_optional(&globals.db)
    .await?;

    let Some(user) = user else {
        return Err(AppError::EntityNotFound("user not found".to_string()));
    };

    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(&user.password_hash).unwrap();

    if argon2
        .verify_password(credentials.password.as_bytes(), &password_hash)
        .is_err()
    {
        return Err(AppError::InvalidCredentials);
    }

    let valid_for = if credentials.extend_session {
        TimeDelta::days(120)
    } else {
        TimeDelta::hours(12)
    };

    let now = Utc::now();
    let expires_at = now + valid_for;

    let mut token = [0; 128];
    let mut token_id: [u8; 64];

    loop {
        OsRng.fill_bytes(&mut token);

        let mut hasher = Sha512::new();
        hasher.update(&token);
        token_id = hasher.finalize().into();

        let res = sqlx::query!(
            "insert into sessions (
                token, token_id, username,
                last_used_at, expires_at, user_agent, ip_address
            ) values ($1, $2, $3, $4, $5, $6, $7)",
            &token,
            &token_id,
            user.username,
            now.naive_utc(),
            expires_at.naive_utc(),
            user_agent,
            ip_address
        )
        .execute(&globals.db)
        .await;

        let db_err = res.as_ref().err().and_then(|e| e.as_database_error());
        if db_err.is_some_and(|e| e.constraint() == Some("token_pkey")) {
            continue;
        }

        res?;
        break;
    }

    Ok(AppJson(Token {
        token: BASE64_STANDARD.encode(token),
        token_id: BASE64_STANDARD.encode(token_id),
        expires_at,
    }))
}

#[derive(Debug)]
pub struct AuthUser {
    pub token: [u8; 128],
    pub username: String,
}

impl FromRequestParts<Globals> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        globals: &Globals,
    ) -> Result<Self, Self::Rejection> {
        let conn_info: ConnectInfo<SocketAddr> = parts.extract().await.unwrap();
        let ip_addr = conn_info.0.ip().to_canonical().to_string();

        let Some(header) = parts.headers.get(AUTHORIZATION) else {
            return Err(AppError::AuthHeaderMissing);
        };

        let Ok(token) = header.to_str() else {
            return Err(AppError::InvalidAuthToken);
        };

        let Ok(token) = BASE64_STANDARD.decode(token) else {
            return Err(AppError::InvalidAuthToken);
        };

        let Ok(token) = <[u8; 128]>::try_from(token) else {
            return Err(AppError::InvalidAuthToken);
        };

        let mut tx = globals.db.begin().await?;

        let query = sqlx::query!(
            "select u.username from sessions s join users u on u.username = s.username where s.token = $1",
            &token
        );
        let Some(record) = query.fetch_optional(&mut *tx).await? else {
            return Err(AppError::InvalidAuthToken);
        };

        let user_agent = parts.headers.get(USER_AGENT).and_then(|v| v.to_str().ok());
        if let Some(user_agent) = user_agent {
            sqlx::query!(
                "update sessions set user_agent = $1 where token = $2",
                user_agent,
                &token
            )
            .execute(&mut *tx)
            .await?;
        };

        sqlx::query!(
            "update sessions set ip_address = $1, last_used_at = $2 where token = $3",
            ip_addr,
            Utc::now().naive_utc(),
            &token
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(AuthUser {
            token,
            username: record.username,
        })
    }
}

async fn get_sessions(
    State(globals): State<Globals>,
    auth_user: AuthUser,
) -> Result<AppJson<Items<Session>>> {
    let records = sqlx::query!(
        "select * from sessions where username = $1 order by last_used_at desc",
        &auth_user.username
    )
    .fetch_all(&globals.db)
    .await?;

    let items = records
        .into_iter()
        .map(|record| Session {
            is_current: record.token == auth_user.token,
            token_id: BASE64_STANDARD.encode(record.token_id),
            user_agent: record.user_agent,
            ip_address: record.ip_address,
            last_used_at: record.last_used_at.and_utc(),
            expires_at: record.expires_at.and_utc(),
        })
        .collect();

    Ok(AppJson(Items { items }))
}

async fn end_current_session(
    State(globals): State<Globals>,
    auth_user: AuthUser,
) -> Result<StatusCode> {
    sqlx::query!("delete from sessions where token = $1", &auth_user.token)
        .execute(&globals.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct EndSessionQuery {
    token_id: String,
}

async fn end_session(
    State(globals): State<Globals>,
    Query(EndSessionQuery { token_id }): Query<EndSessionQuery>,
    auth_user: AuthUser,
) -> Result<StatusCode> {
    let Ok(token_id) = BASE64_STANDARD.decode(token_id) else {
        return Err(AppError::InvalidAuthTokenId);
    };

    let Ok(token_id) = <[u8; 64]>::try_from(token_id) else {
        return Err(AppError::InvalidAuthTokenId);
    };

    let record = sqlx::query!(
        "delete from sessions where username = $1 and token_id = $2 returning 1 as marker",
        &auth_user.username,
        &token_id
    )
    .fetch_optional(&globals.db)
    .await?;

    if record.is_none() {
        return Err(AppError::InvalidAuthTokenId);
    }

    Ok(StatusCode::NO_CONTENT)
}

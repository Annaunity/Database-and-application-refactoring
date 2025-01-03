use std::net::SocketAddr;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::extract::{ConnectInfo, FromRequestParts, State};
use axum::http::HeaderMap;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum::http::request::Parts;
use axum::routing::post;
use axum::{RequestPartsExt, Router};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, TimeDelta, Utc};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;

pub fn routes() -> Router<Globals> {
    Router::new().route("/", post(auth))
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Credentials {
    username_or_email: String,
    password: String,
    extend_session: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Token {
    token: String,
    expires_at: DateTime<Utc>,
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

    let expires_at = Utc::now() + valid_for;

    let mut token = [0; 128];
    loop {
        OsRng.fill_bytes(&mut token);

        let res = sqlx::query!(
            "insert into sessions (token, username, expires_at, user_agent, ip_address) values ($1, $2, $3, $4, $5)",
            &token,
            user.username,
            expires_at.naive_utc(),
            user_agent,
            ip_address
        ).execute(&globals.db).await;

        let db_err = res.as_ref().err().and_then(|e| e.as_database_error());
        if db_err.is_some_and(|e| e.constraint() == Some("token_pkey")) {
            continue;
        }

        res?;
        break;
    }

    Ok(AppJson(Token {
        token: BASE64_STANDARD.encode(token),
        expires_at,
    }))
}

pub struct AuthUser {
    pub username: String,
    pub email: String,
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

        let query = sqlx::query!(
            "select u.username, u.email from sessions s join users u on u.username = s.username where s.token = $1",
            &token
        );
        let Some(record) = query.fetch_optional(&globals.db).await? else {
            return Err(AppError::InvalidAuthToken);
        };

        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown");

        sqlx::query!(
            "update sessions set user_agent = $1, ip_address = $2 where token = $3",
            user_agent,
            ip_addr,
            &token
        )
        .execute(&globals.db)
        .await?;

        Ok(AuthUser {
            username: record.username,
            email: record.email,
        })
    }
}

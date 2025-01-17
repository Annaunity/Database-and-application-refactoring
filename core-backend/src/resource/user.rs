use std::str::FromStr;

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, SaltString};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use rand::rngs::OsRng;

use crate::auth::AuthUser;
use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{CreateUser, FavouriteAnimal, User};

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(create_user))
        .route("/{username}", get(get_user))
}

async fn create_user(
    State(globals): State<Globals>,
    AppJson(user): AppJson<CreateUser>,
) -> Result<StatusCode> {
    if user.username.trim().is_empty() {
        return Err(AppError::InvalidData("invalid username".to_string()));
    }

    if !user.email.contains('@') {
        return Err(AppError::InvalidData("invalid email".to_string()));
    }

    if user.password.trim().is_empty() {
        return Err(AppError::InvalidData("invalid password".to_string()));
    }

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2
        .hash_password(user.password.as_bytes(), &salt)?
        .serialize();

    let res = sqlx::query!(
        "
        insert into users (username, email, password_hash, favourite_animal) values ($1, $2, $3, $4);
        ",
        user.username,
        user.email,
        password_hash.as_str(),
        user.favourite_animal.as_str(),
    )
    .execute(&globals.db)
    .await;

    let db_err = res.as_ref().err().and_then(|e| e.as_database_error());

    if db_err.is_some_and(|e| e.constraint() == Some("users_pkey")) {
        return Err(AppError::EntityExists(format!(
            "username {:#} is already taken",
            user.username
        )));
    }

    if db_err.is_some_and(|e| e.constraint() == Some("users_email_key")) {
        return Err(AppError::EntityExists(format!(
            "email {:#} is already taken",
            user.email
        )));
    }

    res?;

    Ok(StatusCode::CREATED)
}

async fn get_user(
    State(globals): State<Globals>,
    Path(username): Path<String>,
    auth_user: AuthUser,
) -> Result<AppJson<User>> {
    let res = sqlx::query!(
        "
        select username, email, favourite_animal from users where username = $1;
        ",
        if username == "me" {
            &auth_user.username
        } else {
            &username
        },
    )
    .fetch_optional(&globals.db)
    .await?;

    let Some(user) = res else {
        return Err(AppError::EntityNotFound(format!(
            "user {username:?} doesn't exist"
        )));
    };

    let favourite_animal = FavouriteAnimal::from_str(&user.favourite_animal)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = User {
        username: user.username,
        email: user.email,
        favourite_animal,
    };

    Ok(AppJson(user))
}

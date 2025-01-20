use std::str::FromStr;

use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier as _};
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use rand::rngs::OsRng;

use crate::auth::AuthUser;
use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{FavouriteAnimal, NewUser, UpdateUser, User};

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(create_user))
        .route("/{username}", get(get_user))
        .route("/{username}", patch(update_user))
}

async fn create_user(
    State(globals): State<Globals>,
    AppJson(user): AppJson<NewUser>,
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

async fn update_user(
    State(globals): State<Globals>,
    Path(username): Path<String>,
    auth_user: AuthUser,
    AppJson(update): AppJson<UpdateUser>,
) -> Result<AppJson<User>> {
    let username = if username == "me" {
        &auth_user.username
    } else {
        &username
    };

    if username != &auth_user.username {
        return Err(AppError::Unauthorized("permission denied".to_string()));
    }

    let mut tx = globals.db.begin().await?;

    if let Some(email) = update.email {
        sqlx::query!(
            "update users set email = $1 where username = $2",
            email,
            username
        )
        .execute(&mut *tx)
        .await?;
    }

    if let Some(favourite_animal) = update.favourite_animal {
        sqlx::query!(
            "update users set favourite_animal = $1 where username = $2",
            favourite_animal.as_str(),
            username
        )
        .execute(&mut *tx)
        .await?;
    }

    if let Some(update_password) = update.update_password {
        let record = sqlx::query!(
            "select password_hash from users where username = $1",
            username
        )
        .fetch_one(&globals.db)
        .await?;
        let password_hash = record.password_hash;

        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&password_hash).unwrap();

        if argon2
            .verify_password(update_password.old_password.as_bytes(), &password_hash)
            .is_err()
        {
            return Err(AppError::InvalidCredentials);
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(update_password.new_password.as_bytes(), &salt)?
            .serialize();

        sqlx::query!(
            "update users set password_hash = $1 where username = $2",
            password_hash.as_str(),
            username
        )
        .execute(&mut *tx)
        .await?;
    }

    let record = sqlx::query!("select * from users where username = $1", username)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    let favourite_animal = FavouriteAnimal::from_str(&record.favourite_animal)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = User {
        username: record.username,
        email: record.email,
        favourite_animal,
    };

    Ok(AppJson(user))
}

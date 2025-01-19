use std::fmt::Display;

use axum::extract::FromRequest;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};

pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Internal(String),
    #[error("{0}")]
    InvalidData(String),
    #[error("{0}")]
    EntityExists(String),
    #[error("{0}")]
    EntityNotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("auth header missing")]
    AuthHeaderMissing,
    #[error("invalid auth token")]
    InvalidAuthToken,
    #[error("invalid auth token id")]
    InvalidAuthTokenId,
    #[error("invalid json")]
    JsonRejection(#[from] JsonRejection),
    #[error("database error")]
    DbError(#[from] sqlx::Error),
    #[error("password hashing error")]
    PasswordHashingError(#[from] argon2::password_hash::Error),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    pub fn message<T: Display>(error: T) -> ErrorResponse {
        ErrorResponse {
            message: error.to_string(),
            extra: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new<T: std::error::Error>(error: T) -> ErrorResponse {
        ErrorResponse {
            message: error.to_string(),
            extra: Some(format!("{error:#?}")),
            timestamp: Utc::now(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidData(_) => StatusCode::BAD_REQUEST,
            AppError::EntityExists(_) => StatusCode::CONFLICT,
            AppError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::AuthHeaderMissing => StatusCode::UNAUTHORIZED,
            AppError::InvalidAuthToken => StatusCode::UNAUTHORIZED,
            AppError::InvalidAuthTokenId => StatusCode::UNAUTHORIZED,
            AppError::JsonRejection(error) => error.status(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, AppJson(ErrorResponse::new(self))).into_response()
    }
}

use std::fmt::Display;

use axum::extract::FromRequest;
use axum::extract::multipart::MultipartError;
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
    EntityNotFound(String),
    #[error("invalid json")]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    MultipartError(#[from] MultipartError),
    #[error("internal error")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            AppError::MultipartError(e) => e.status(),
            AppError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            AppError::JsonRejection(error) => error.status(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, AppJson(ErrorResponse::new(self))).into_response()
    }
}

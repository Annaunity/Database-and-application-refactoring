mod auth;
mod error;
mod globals;
pub mod model;
mod resource;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::extract::{MatchedPath, Request};
use axum::http::StatusCode;
use axum::{BoxError, Router};
use image_backend::ImageService;
use sqlx::{Pool, Postgres};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::error::{AppJson, ErrorResponse};
use crate::globals::Globals;

async fn handle_timeout_error(err: BoxError) -> (StatusCode, AppJson<ErrorResponse>) {
    if err.is::<tower::timeout::error::Elapsed>() {
        return (
            StatusCode::REQUEST_TIMEOUT,
            AppJson(ErrorResponse::message(err)),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        AppJson(ErrorResponse::message(err)),
    )
}

pub fn build_app(db: Pool<Postgres>) -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
    let globals = Globals {
        image_service: Arc::new(ImageService::new("http://127.0.0.1:2024".to_string())),
        db,
    };

    let api = Router::new()
        .nest("/auth", auth::routes())
        .nest("/user", resource::user::routes())
        .nest("/drawing", resource::drawing::routes());

    Router::new()
        .nest("/api/v1", api)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(5)),
        )
        .layer(TraceLayer::new_for_http().make_span_with(|req: &Request| {
            let method = req.method();
            let uri = req.uri();

            let matched_path = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched_path| matched_path.as_str());

            tracing::debug_span!("request", %method, %uri, matched_path)
        }))
        .with_state(globals)
        .into_make_service_with_connect_info::<SocketAddr>()
}

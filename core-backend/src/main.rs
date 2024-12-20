mod auth;
mod error;
mod globals;
mod model;
mod resource;

use std::net::SocketAddr;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::{MatchedPath, Request};
use axum::http::StatusCode;
use axum::{BoxError, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting core backend");

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let globals = Globals {
        db: sqlx::Pool::connect(&database_url).await.unwrap(),
    };

    let api = Router::new()
        .nest("/auth", auth::routes())
        .nest("/user", resource::user::routes());

    let app = Router::new()
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
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2004").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

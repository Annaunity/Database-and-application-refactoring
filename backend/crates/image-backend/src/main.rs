use std::path::PathBuf;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

    tracing::info!("Starting image backend");

    let data_path = PathBuf::from(std::env::var("DATA_PATH").unwrap());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2024").await.unwrap();

    let app = image_backend::build_app(&data_path);

    axum::serve(listener, app).await.unwrap();
}

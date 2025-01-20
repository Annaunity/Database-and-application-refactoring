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

    tracing::info!("Starting core backend");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2004").await.unwrap();

    let database_url = std::env::var("DATABASE_URL").unwrap();
    let db = sqlx::Pool::connect(&database_url).await.unwrap();

    let app = core_backend::build_app(db);

    axum::serve(listener, app).await.unwrap();
}

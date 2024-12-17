#[derive(Clone)]
pub struct Globals {
    pub db: sqlx::Pool<sqlx::Postgres>,
}

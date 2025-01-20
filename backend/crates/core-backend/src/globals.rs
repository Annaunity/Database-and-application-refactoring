use std::sync::Arc;

use image_backend::ImageService;

#[derive(Clone)]
pub struct Globals {
    pub image_service: Arc<ImageService>,
    pub db: sqlx::Pool<sqlx::Postgres>,
}

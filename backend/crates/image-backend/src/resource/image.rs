use std::io::Cursor;

use axum::Router;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use futures_util::StreamExt as _;
use image::{ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;

use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{ImageId, UploadResult};

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024;

pub fn routes() -> Router<Globals> {
    Router::new()
        .route(
            "/",
            post(upload_image).layer(DefaultBodyLimit::max(MAX_IMAGE_SIZE)),
        )
        .route("/{id}", get(get_image))
        .route("/{id}", delete(delete_image))
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct UploadQuery {
    width: Option<u32>,
    height: Option<u32>,
}

async fn upload_image(
    State(globals): State<Globals>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<AppJson<UploadResult>> {
    let Some(field) = multipart.next_field().await? else {
        return Err(AppError::InvalidData("no image uploaded".to_string()));
    };

    if field.content_type() != Some("image/png") {
        return Err(AppError::InvalidData("unsupported image type".to_string()));
    }

    let bytes = field.bytes().await?.to_vec();
    let reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);

    let image = reader
        .decode()
        .map_err(|_| AppError::InvalidData("invalid image".to_string()))?;

    let image = image.into_rgb8();

    if let Some(width) = query.width {
        if width != image.width() {
            return Err(AppError::InvalidData("invalid width".to_string()));
        }
    }

    if let Some(height) = query.height {
        if height != image.height() {
            return Err(AppError::InvalidData("invalid height".to_string()));
        }
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"v0");
    hasher.update(&image.width().to_le_bytes());
    hasher.update(&image.height().to_le_bytes());
    hasher.update(&image.as_raw());
    let hash = hasher.finalize();

    let id = ImageId(format!("0{}", hash.to_hex()));

    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    if tokio::fs::try_exists(&path).await? {
        return Err(AppError::EntityExists("image exists".to_string()));
    }

    let mut bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|_| AppError::Internal("failed to encode image".to_string()))?;

    tokio::fs::write(&path, bytes).await?;

    Ok(AppJson(UploadResult { id }))
}

async fn get_image(
    State(globals): State<Globals>,
    Path(id): Path<ImageId>,
) -> Result<(HeaderMap, Body)> {
    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_| AppError::EntityNotFound("image not found".to_string()))?;

    let stream = ReaderStream::new(file).map(|v| {
        v.map_err(|error| {
            tracing::error!(%error, "read error: ");
            AppError::Internal("internal error".to_string())
        })
    });

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "image/png".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}.png\"", id.0)
            .parse()
            .unwrap(),
    );

    let body = Body::from_stream(stream);

    Ok((headers, body))
}

async fn delete_image(
    State(globals): State<Globals>,
    Path(id): Path<ImageId>,
) -> Result<StatusCode> {
    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    if !tokio::fs::try_exists(&path).await? {
        return Err(AppError::EntityNotFound("image not found".to_string()));
    }

    tokio::fs::remove_file(&path).await?;

    Ok(StatusCode::NO_CONTENT)
}

use std::io::Cursor;

use axum::Router;
use axum::body::Body;
use axum::extract::multipart::MultipartRejection;
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use futures_util::StreamExt as _;
use image::imageops::FilterType;
use image::{ImageFormat, ImageReader, Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
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
        .route("/{id}/resize", post(resize_image))
        .route("/{id}", get(get_image))
        .route("/{id}", delete(delete_image))
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct UploadQuery {
    width: Option<u32>,
    height: Option<u32>,
}

#[axum::debug_handler]
async fn upload_image(
    State(globals): State<Globals>,
    Query(query): Query<UploadQuery>,
    mut multipart: Result<Multipart, MultipartRejection>,
) -> Result<AppJson<UploadResult>> {
    let multipart_field = match multipart.as_mut() {
        Ok(v) => v.next_field().await?,
        Err(_) => None,
    };

    let image = match multipart_field {
        Some(field) => {
            if field.content_type() != Some("image/png") {
                return Err(AppError::InvalidData("unsupported image type".to_string()));
            }

            let bytes = field.bytes().await?;

            spawn_blocking(move || {
                let reader =
                    ImageReader::with_format(Cursor::new(bytes.to_vec()), ImageFormat::Png);

                let image = reader
                    .decode()
                    .map_err(|_| AppError::InvalidData("invalid image".to_string()))?;

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

                Ok(image.into_rgb8())
            })
            .await
            .unwrap()?
        }
        _ => {
            let Some(width) = query.width else {
                return Err(AppError::InvalidData(
                    "width is required without body".to_string(),
                ));
            };

            let Some(height) = query.height else {
                return Err(AppError::InvalidData(
                    "height is required without body".to_string(),
                ));
            };

            spawn_blocking(move || {
                let mut image = RgbImage::new(width, height);
                for pixel in image.pixels_mut() {
                    *pixel = Rgb([255, 255, 255]);
                }
                image
            })
            .await
            .unwrap()
        }
    };

    let (id, bytes) = encode_image(image).await?;

    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    if !tokio::fs::try_exists(&path).await? {
        tokio::fs::write(&path, bytes).await?;
    }

    Ok(AppJson(UploadResult { id }))
}

async fn encode_image(image: RgbImage) -> Result<(ImageId, Vec<u8>)> {
    spawn_blocking(move || {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"v0");
        hasher.update(&image.width().to_le_bytes());
        hasher.update(&image.height().to_le_bytes());
        hasher.update(&image.as_raw());
        hasher.finalize();
        let hash = hasher.finalize();

        let mut bytes: Vec<u8> = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .map_err(|_| AppError::Internal("failed to encode image".to_string()))?;

        let id = ImageId(format!("0{}", hash.to_hex()));

        Ok::<_, AppError>((id, bytes))
    })
    .await
    .unwrap()
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct ResizeQuery {
    width: u32,
    height: u32,
}

async fn resize_image(
    State(globals): State<Globals>,
    Path(id): Path<ImageId>,
    Query(query): Query<ResizeQuery>,
) -> Result<AppJson<UploadResult>> {
    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    if !tokio::fs::try_exists(&path).await? {
        return Err(AppError::EntityNotFound("image not found".to_string()));
    }

    let data = tokio::fs::read(&path).await?;

    let image = spawn_blocking(move || {
        let reader = ImageReader::with_format(Cursor::new(data.to_vec()), ImageFormat::Png);

        let image = reader
            .decode()
            .map_err(|_| AppError::Internal("invalid image".to_string()))?;

        let resized = image
            .resize(query.width, query.height, FilterType::Lanczos3)
            .to_rgb8();

        Ok::<_, AppError>(resized)
    })
    .await
    .unwrap()?;

    let (id, data) = encode_image(image).await?;

    let mut path = globals.data_path.join(&id.0);
    path.set_extension("png");

    if !tokio::fs::try_exists(&path).await? {
        tokio::fs::write(&path, data).await?;
    }

    Ok(AppJson(UploadResult { id }))
}

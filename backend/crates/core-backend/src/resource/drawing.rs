use axum::Router;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post, put};
use chrono::Utc;
use image_backend::model::ImageId;
use serde::{Deserialize, Serialize};

use crate::auth::AuthUser;
use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{Drawing, DrawingVersion, Items, NewDrawing};

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(create_drawing))
        .route("/owned", get(get_owned_drawings))
        .route("/{id}", get(get_drawing))
        .route("/{id}", delete(delete_drawing))
        .route("/{id}/version", get(get_versions))
        .route("/{id}/version/{version_id}", get(get_version))
        .route("/{id}/version/latest", put(upload_new_version))
        .route("/{id}/version/latest", get(get_latest_version))
}

async fn create_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    AppJson(new_drawing): AppJson<NewDrawing>,
) -> Result<(StatusCode, AppJson<Drawing>)> {
    if new_drawing.name.trim().is_empty() {
        return Err(AppError::InvalidData("invalid name".to_string()));
    }

    if new_drawing.width < 1 {
        return Err(AppError::InvalidData("invalid width".to_string()));
    }

    if new_drawing.height < 1 {
        return Err(AppError::InvalidData("invalid height".to_string()));
    }

    if new_drawing.width > 2048 {
        return Err(AppError::InvalidData("width too large".to_string()));
    }

    if new_drawing.height > 2048 {
        return Err(AppError::InvalidData("height too large".to_string()));
    }

    let now = Utc::now();

    let image_id = globals
        .image_service
        .create_white_image(new_drawing.width as u32, new_drawing.height as u32)
        .await?
        .id;

    let thumbnail_image_id = globals
        .image_service
        .resize_image(image_id.clone(), 256, 256)
        .await?
        .id;

    let query = sqlx::query!(
        "insert into drawings (
            name, owner, width, height, image_id,
            thumbnail_image_id, created_at, updated_at)
        values ($1, $2, $3, $4, $5, $6, $7, $8) returning *",
        new_drawing.name,
        auth_user.username,
        new_drawing.width,
        new_drawing.height,
        image_id.0,
        thumbnail_image_id.0,
        now.naive_utc(),
        now.naive_utc(),
    );

    let record = query.fetch_one(&globals.db).await?;

    let drawing = Drawing {
        id: record.id,
        name: record.name,
        width: record.width,
        height: record.height,
        created_at: record.created_at.and_utc(),
        updated_at: record.updated_at.and_utc(),
    };

    Ok((StatusCode::CREATED, AppJson(drawing)))
}

async fn get_owned_drawings(
    State(globals): State<Globals>,
    auth_user: AuthUser,
) -> Result<AppJson<Items<Drawing>>> {
    let query = sqlx::query!(
        "select * from drawings where owner = $1 order by updated_at desc",
        auth_user.username,
    );

    let records = query.fetch_all(&globals.db).await?;

    let items = records
        .into_iter()
        .map(|record| Drawing {
            id: record.id,
            name: record.name,
            width: record.width,
            height: record.height,
            created_at: record.created_at.and_utc(),
            updated_at: record.updated_at.and_utc(),
        })
        .collect();

    Ok(AppJson(Items { items }))
}

async fn get_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<AppJson<Drawing>> {
    let query = sqlx::query!(
        "select
            name, owner, width, height, image_id,
            thumbnail_image_id, created_at, updated_at
        from drawings
        where id = $1",
        id
    );

    let Some(record) = query.fetch_optional(&globals.db).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let drawing = Drawing {
        id,
        name: record.name,
        width: record.width,
        height: record.height,
        created_at: record.created_at.and_utc(),
        updated_at: record.updated_at.and_utc(),
    };

    Ok(AppJson(drawing))
}

async fn delete_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<StatusCode> {
    let mut tx = globals.db.begin().await?;

    let query = sqlx::query!("select owner from drawings where id = $1", id);
    let Some(record) = query.fetch_optional(&mut *tx).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let query = sqlx::query!("delete from drawings where id = $1", id);
    query.execute(&mut *tx).await?;

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_versions(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<AppJson<Items<DrawingVersion>>> {
    let mut tx = globals.db.begin().await?;

    let query = sqlx::query!("select owner from drawings where id = $1", id);
    let Some(record) = query.fetch_optional(&mut *tx).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let rows = sqlx::query!(
        "select * from drawing_versions where drawing_id = $1 order by created_at desc",
        id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    let items = rows
        .into_iter()
        .map(|record| DrawingVersion {
            id: record.version_id,
            width: record.width,
            height: record.height,
            created_at: record.created_at.and_utc(),
        })
        .collect();

    Ok(AppJson(Items { items }))
}

async fn upload_new_version(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<StatusCode> {
    let Some(field) = multipart.next_field().await? else {
        return Err(AppError::InvalidData("no image provided".to_string()));
    };

    if field.content_type() != Some("image/png") {
        return Err(AppError::InvalidData("unsupported image type".to_string()));
    }

    let mut tx = globals.db.begin().await?;

    let query = sqlx::query!("select * from drawings where id = $1", id);
    let Some(record) = query.fetch_optional(&mut *tx).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let data = field.bytes().await?.to_vec();

    let upload = globals
        .image_service
        .create_image(record.width as u32, record.height as u32, data)
        .await?;

    let thumbnail_upload = globals
        .image_service
        .resize_image(upload.id.clone(), 256, 256)
        .await?;

    sqlx::query!(
        "update drawings set image_id = $1, thumbnail_image_id = $2, updated_at = $3 where id = $4",
        upload.id.0,
        thumbnail_upload.id.0,
        Utc::now().naive_utc(),
        id
    )
    .execute(&mut *tx)
    .await?;

    let num_versions = sqlx::query!(
        "select count(*) from drawing_versions where drawing_id = $1",
        id
    )
    .fetch_one(&mut *tx)
    .await?
    .count
    .unwrap_or(0);

    sqlx::query!(
        "insert into drawing_versions (
            drawing_id, version_id, width, height, image_id, thumbnail_image_id, created_at)
        values ($1, $2, $3, $4, $5, $6, $7)",
        id,
        (num_versions as i32) + 1,
        record.width,
        record.height,
        upload.id.0,
        thumbnail_upload.id.0,
        Utc::now().naive_utc()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct GetLatestVersionQuery {
    #[serde(default)]
    thumbnail: bool,
}

async fn get_latest_version(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
    Query(query_params): Query<GetLatestVersionQuery>,
) -> Result<(HeaderMap, Vec<u8>)> {
    let query = sqlx::query!("select * from drawings where id = $1", id);
    let Some(record) = query.fetch_optional(&globals.db).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let id = if query_params.thumbnail {
        record.thumbnail_image_id
    } else {
        record.image_id
    };

    let image = globals.image_service.get_image(ImageId(id)).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "image/png".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        "attachment; filename=\"drawing.png\"".parse().unwrap(),
    );

    Ok((headers, image))
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct GetVersionQuery {
    #[serde(default)]
    thumbnail: bool,
}

async fn get_version(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path((id, version_id)): Path<(i32, i32)>,
    Query(query_params): Query<GetLatestVersionQuery>,
) -> Result<(HeaderMap, Vec<u8>)> {
    let query = sqlx::query!("select * from drawings where id = $1", id);
    let Some(record) = query.fetch_optional(&globals.db).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "drawing not found".to_string(),
        ));
    };

    if auth_user.username != record.owner {
        return Err(crate::error::AppError::Unauthorized(
            "drawing not owned by the user".to_string(),
        ));
    }

    let query = sqlx::query!(
        "select * from drawing_versions where drawing_id = $1 and version_id = $2",
        id,
        version_id
    );

    let Some(record) = query.fetch_optional(&globals.db).await? else {
        return Err(crate::error::AppError::EntityNotFound(
            "version not found".to_string(),
        ));
    };

    let id = if query_params.thumbnail {
        record.thumbnail_image_id
    } else {
        record.image_id
    };

    let image = globals.image_service.get_image(ImageId(id)).await?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "image/png".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        "attachment; filename=\"drawing.png\"".parse().unwrap(),
    );

    Ok((headers, image))
}

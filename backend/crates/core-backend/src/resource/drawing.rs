use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use chrono::Utc;

use crate::auth::AuthUser;
use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;
use crate::model::{Drawing, Items, NewDrawing};

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(create_drawing))
        .route("/owned", get(get_owned_drawings))
        .route("/{id}", get(get_drawing))
        .route("/{id}", delete(delete_drawing))
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

    let now = Utc::now();

    let query = sqlx::query!(
        "insert into drawings (
            name, owner, width, height, image_id,
            thumbnail_image_id, created_at, updated_at)
        values ($1, $2, $3, $4, $5, $6, $7, $8) returning *",
        new_drawing.name,
        auth_user.username,
        new_drawing.width,
        new_drawing.height,
        "TODO",
        "TODO",
        now.naive_utc(),
        now.naive_utc(),
    );

    let record = query.fetch_one(&globals.db).await?;

    let drawing = Drawing {
        id: record.id,
        name: record.name,
        width: record.width,
        height: record.height,
        image_id: record.image_id,
        thumbnail_image_id: record.thumbnail_image_id,
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
        "select
            id, name, owner, width, height, image_id,
            thumbnail_image_id, created_at, updated_at
        from drawings
        where owner = $1
        order by updated_at desc",
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
            image_id: record.image_id,
            thumbnail_image_id: record.thumbnail_image_id,
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
        image_id: record.image_id,
        thumbnail_image_id: record.thumbnail_image_id,
        created_at: record.created_at.and_utc(),
        updated_at: record.updated_at.and_utc(),
    };

    Ok(AppJson(drawing))
}

async fn delete_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<()> {
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

    Ok(())
}

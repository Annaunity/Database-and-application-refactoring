use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};

use crate::auth::AuthUser;
use crate::error::{AppError, AppJson, Result};
use crate::globals::Globals;

pub fn routes() -> Router<Globals> {
    Router::new()
        .route("/", post(create_drawing))
        .route("/owned", get(get_owned_drawings))
        .route("/{id}", get(get_drawing))
        .route("/{id}", delete(delete_drawing))
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Drawing {
    id: i32,
    name: String,
    width: i32,
    height: i32,
    image_id: String,
    thumbnail_image_id: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewDrawing {
    name: String,
    width: i32,
    height: i32,
}

async fn create_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    AppJson(new_drawing): AppJson<NewDrawing>,
) -> Result<AppJson<Drawing>> {
    if new_drawing.name.trim().is_empty() {
        return Err(AppError::InvalidData("invalid name".to_string()));
    }

    if new_drawing.width < 1 {
        return Err(AppError::InvalidData("invalid width".to_string()));
    }

    if new_drawing.height < 1 {
        return Err(AppError::InvalidData("invalid height".to_string()));
    }

    let query = sqlx::query!(
        "insert into drawings (name, owner, width, height, image_id, thumbnail_image_id) values ($1, $2, $3, $4, $5, $6) returning id",
        new_drawing.name,
        auth_user.username,
        new_drawing.width,
        new_drawing.height,
        "TODO",
        "TODO"
    );

    let record = query.fetch_one(&globals.db).await?;
    let id = record.id;

    let drawing = Drawing {
        id,
        name: new_drawing.name,
        width: new_drawing.width,
        height: new_drawing.height,
        image_id: "TODO".to_string(),
        thumbnail_image_id: "TODO".to_string(),
    };

    Ok(AppJson(drawing))
}

#[derive(Debug, Clone, serde::Serialize)]
struct OwnedDrawings {
    items: Vec<Drawing>,
}

#[axum::debug_handler]
async fn get_owned_drawings(
    State(globals): State<Globals>,
    auth_user: AuthUser,
) -> Result<AppJson<OwnedDrawings>> {
    let query = sqlx::query!(
        "select id, name, owner, width, height, image_id, thumbnail_image_id from drawings where owner = $1",
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
        })
        .collect();

    let drawings = OwnedDrawings { items };

    Ok(AppJson(drawings))
}

#[axum::debug_handler]
async fn get_drawing(
    State(globals): State<Globals>,
    auth_user: AuthUser,
    Path(id): Path<i32>,
) -> Result<AppJson<Drawing>> {
    let query = sqlx::query!(
        "select name, owner, width, height, image_id, thumbnail_image_id from drawings where id = $1",
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
    };

    Ok(AppJson(drawing))
}

#[axum::debug_handler]
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

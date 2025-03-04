use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum_test::TestServer;
use axum_test::multipart::{MultipartForm, Part};
use chrono::{TimeDelta, Utc};
use core_backend::model::{Drawing, Items, NewDrawing, Token};
use sqlx::PgPool;

use crate::user::TestUser;

pub struct TestDrawing {
    pub name: &'static str,
    pub width: i32,
    pub height: i32,
}

impl TestDrawing {
    pub const SHARK: Self = Self {
        name: "shark",
        width: 800,
        height: 600,
    };

    pub const MIKU: Self = Self {
        name: "miku",
        width: 1920,
        height: 1080,
    };

    pub const KITTEN: Self = Self {
        name: "kitteh:3",
        width: 512, // must match size of kitten.png
        height: 512,
    };

    pub fn as_new_drawing(&self) -> NewDrawing {
        NewDrawing {
            name: self.name.to_string(),
            width: self.width,
            height: self.height,
        }
    }

    pub async fn create(&self, server: &TestServer, token: &Token) -> Drawing {
        server
            .post("/api/v1/drawing")
            .add_header(AUTHORIZATION, &token.token)
            .json(&self.as_new_drawing())
            .await
            .json()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_drawing(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let now = Utc::now();

    let res = server
        .post("/api/v1/drawing")
        .add_header(AUTHORIZATION, &token.token)
        .json(&TestDrawing::SHARK.as_new_drawing())
        .await;

    res.assert_status(StatusCode::CREATED);

    let drawing: Drawing = res.json();
    assert_eq!(drawing.name, TestDrawing::SHARK.name);
    assert_eq!(drawing.width, TestDrawing::SHARK.width);
    assert_eq!(drawing.height, TestDrawing::SHARK.height);
    assert_eq!(drawing.created_at, drawing.updated_at);
    assert!((drawing.created_at - now) < TimeDelta::seconds(1));
    assert!((drawing.updated_at - now) < TimeDelta::seconds(1));
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_drawing(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let drawing = TestDrawing::SHARK.create(&server, &token).await;

    let res = server
        .get(&format!("/api/v1/drawing/{}", drawing.id))
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_status_ok();
    res.assert_json(&drawing);
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_owned_drawing(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let shark = TestDrawing::SHARK.create(&server, &token).await;
    let miku = TestDrawing::MIKU.create(&server, &token).await;

    let res = server
        .get(&format!("/api/v1/drawing/owned"))
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_status_ok();
    res.assert_json(&Items {
        items: vec![miku, shark], // newest first
    });
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_drawing(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let drawing = TestDrawing::SHARK.create(&server, &token).await;

    let res = server
        .delete(&format!("/api/v1/drawing/{}", drawing.id))
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_status(StatusCode::NO_CONTENT);

    let res = server
        .get(&format!("/api/v1/drawing/{}", drawing.id))
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_status_not_found();
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_latest_version(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let drawing = TestDrawing::SHARK.create(&server, &token).await;

    let res = server
        .get(&format!("/api/v1/drawing/{}/version/latest", drawing.id))
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_status(StatusCode::OK);
    res.assert_header(CONTENT_TYPE, "image/png");
}

#[sqlx::test(migrations = "../../migrations")]
async fn upload_new_version(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let drawing = TestDrawing::KITTEN.create(&server, &token).await;

    let image = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-data/kitten.png"
    ))
    .unwrap();

    let res = server
        .put(&format!("/api/v1/drawing/{}/version/latest", drawing.id))
        .add_header(AUTHORIZATION, &token.token)
        .multipart(
            MultipartForm::new().add_part("image", Part::bytes(image).mime_type("image/png")),
        )
        .await;

    res.assert_status(StatusCode::NO_CONTENT);
}

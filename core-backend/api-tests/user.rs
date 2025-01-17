use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum_test::TestServer;
use sqlx::PgPool;

use crate::utils::TestUser;

#[sqlx::test(migrations = "../migrations")]
async fn create_user(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();

    let res = server
        .post("/api/v1/user")
        .json(&TestUser::ALEX.as_create_user())
        .await;

    res.assert_status(StatusCode::CREATED);
    res.assert_text("");
}

#[sqlx::test(migrations = "../migrations")]
async fn get_user(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let res = server
        .get("/api/v1/user/alex")
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_json(&TestUser::ALEX.as_user());
}

#[sqlx::test(migrations = "../migrations")]
async fn get_user_me(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_json(&TestUser::ALEX.as_user());
}

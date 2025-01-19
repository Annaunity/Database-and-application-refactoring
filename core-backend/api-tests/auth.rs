use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_test::TestServer;
use core_backend::model::{Credentials, Items, Session, Token};
use sqlx::PgPool;

use crate::user::TestUser;

#[sqlx::test(migrations = "../migrations")]
async fn auth_via_username(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    TestUser::ALEX.create(&server).await;

    let res = server
        .post("/api/v1/auth")
        .json(&Credentials {
            username_or_email: TestUser::ALEX.username.to_string(),
            password: TestUser::ALEX.password.to_string(),
            extend_session: false,
        })
        .await;

    res.assert_status_ok();

    let token: Token = res.json();

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "../migrations")]
async fn auth_via_email(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    TestUser::ALEX.create(&server).await;

    let res = server
        .post("/api/v1/auth")
        .json(&Credentials {
            username_or_email: TestUser::ALEX.email.to_string(),
            password: TestUser::ALEX.password.to_string(),
            extend_session: false,
        })
        .await;

    res.assert_status_ok();

    let token: Token = res.json();

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status_ok();
}

#[sqlx::test(migrations = "../migrations")]
async fn end_current_session(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let res = server
        .delete("/api/v1/auth")
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status(StatusCode::NO_CONTENT);

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status_unauthorized();
}

#[sqlx::test(migrations = "../migrations")]
async fn end_session_by_token_id(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();
    let token = TestUser::ALEX.create_and_auth(&server).await;

    let res = server
        .delete("/api/v1/auth/session")
        .add_query_param("token_id", token.token_id)
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status(StatusCode::NO_CONTENT);

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;
    res.assert_status_unauthorized();
}

#[sqlx::test(migrations = "../migrations")]
async fn list_sessions(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();

    TestUser::ALEX.create(&server).await;

    let credentials = Credentials {
        username_or_email: TestUser::ALEX.username.to_string(),
        password: TestUser::ALEX.password.to_string(),
        extend_session: false,
    };

    let token_1 = server
        .post("/api/v1/auth")
        .json(&credentials)
        .add_header(USER_AGENT, "1")
        .await
        .json::<Token>();

    let token_2 = server
        .post("/api/v1/auth")
        .json(&credentials)
        .add_header(USER_AGENT, "2")
        .await
        .json::<Token>();

    let res = server
        .get("/api/v1/auth/session")
        .add_header(AUTHORIZATION, &token_1.token)
        .await;
    res.assert_status_ok();

    let sessions = res.json::<Items<Session>>();

    assert_eq!(sessions.items.len(), 2);

    assert_eq!(sessions.items[0].is_current, true);
    assert_eq!(sessions.items[0].token_id, token_1.token_id);
    assert_eq!(sessions.items[0].user_agent, "1");
    assert_eq!(sessions.items[0].ip_address, "127.0.0.1");

    assert_eq!(sessions.items[1].is_current, false);
    assert_eq!(sessions.items[1].token_id, token_2.token_id);
    assert_eq!(sessions.items[1].user_agent, "2");
    assert_eq!(sessions.items[1].ip_address, "127.0.0.1");
}

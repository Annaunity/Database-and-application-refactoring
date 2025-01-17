use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum_test::TestServer;
use core_backend::model::{CreateUser, Credentials, FavouriteAnimal, Token, User};
use sqlx::PgPool;

#[sqlx::test]
async fn create_user(db: PgPool) -> anyhow::Result<()> {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app)?;

    let res = server
        .post("/api/v1/user")
        .json(&CreateUser {
            username: "alex".to_string(),
            email: "alex@nyaalex.site".to_string(),
            password: "password123".to_string(),
            favourite_animal: FavouriteAnimal::Cat,
        })
        .await;

    res.assert_status(StatusCode::CREATED);
    res.assert_text("");

    Ok(())
}

#[sqlx::test]
async fn get_user(db: PgPool) -> anyhow::Result<()> {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app)?;

    let res = server
        .post("/api/v1/user")
        .json(&CreateUser {
            username: "alex".to_string(),
            email: "alex@nyaalex.site".to_string(),
            password: "password123".to_string(),
            favourite_animal: FavouriteAnimal::Cat,
        })
        .await;

    res.assert_status(StatusCode::CREATED);
    res.assert_text("");

    let res = server
        .post("/api/v1/auth")
        .json(&Credentials {
            username_or_email: "alex".to_string(),
            password: "password123".to_string(),
            extend_session: false,
        })
        .await;

    res.assert_status_ok();
    let token: Token = res.json();

    let res = server
        .get("/api/v1/user/alex")
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_json(&User {
        username: "alex".to_string(),
        email: "alex@nyaalex.site".to_string(),
        favourite_animal: FavouriteAnimal::Cat,
    });

    let res = server
        .get("/api/v1/user/me")
        .add_header(AUTHORIZATION, &token.token)
        .await;

    res.assert_json(&User {
        username: "alex".to_string(),
        email: "alex@nyaalex.site".to_string(),
        favourite_animal: FavouriteAnimal::Cat,
    });

    Ok(())
}

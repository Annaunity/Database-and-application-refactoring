use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum_test::TestServer;
use core_backend::model::{Credentials, FavouriteAnimal, NewUser, Token, User};
use sqlx::PgPool;

pub struct TestUser {
    pub username: &'static str,
    pub email: &'static str,
    pub password: &'static str,
    pub favourite_animal: FavouriteAnimal,
}

impl TestUser {
    pub const ALEX: Self = Self {
        username: "alex",
        email: "alex@nyaalex.site",
        password: "password123",
        favourite_animal: FavouriteAnimal::Cat,
    };

    pub fn as_user(&self) -> User {
        User {
            username: self.username.to_string(),
            email: self.email.to_string(),
            favourite_animal: self.favourite_animal,
        }
    }

    pub fn as_new_user(&self) -> NewUser {
        NewUser {
            username: self.username.to_string(),
            email: self.email.to_string(),
            password: self.password.to_string(),
            favourite_animal: self.favourite_animal,
        }
    }

    pub async fn create(&self, server: &TestServer) {
        let res = server
            .post("/api/v1/user")
            .json(&NewUser {
                username: "alex".to_string(),
                email: "alex@nyaalex.site".to_string(),
                password: "password123".to_string(),
                favourite_animal: FavouriteAnimal::Cat,
            })
            .await;

        res.assert_status(StatusCode::CREATED);
        res.assert_text("");
    }

    pub async fn auth(&self, server: &TestServer) -> Token {
        server
            .post("/api/v1/auth")
            .json(&Credentials {
                username_or_email: self.username.to_string(),
                password: self.password.to_string(),
                extend_session: false,
            })
            .await
            .json()
    }

    pub async fn create_and_auth(&self, server: &TestServer) -> Token {
        self.create(server).await;
        self.auth(server).await
    }
}

#[sqlx::test(migrations = "../migrations")]
async fn create_user(db: PgPool) {
    let app = core_backend::build_app(db);
    let server = TestServer::new(app).unwrap();

    let res = server
        .post("/api/v1/user")
        .json(&TestUser::ALEX.as_new_user())
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

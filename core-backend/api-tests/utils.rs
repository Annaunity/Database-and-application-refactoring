use axum::http::StatusCode;
use axum_test::TestServer;
use core_backend::model::{CreateUser, Credentials, FavouriteAnimal, Token, User};

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

    pub fn as_create_user(&self) -> CreateUser {
        CreateUser {
            username: self.username.to_string(),
            email: self.email.to_string(),
            password: self.password.to_string(),
            favourite_animal: self.favourite_animal,
        }
    }

    pub async fn create(&self, server: &TestServer) {
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

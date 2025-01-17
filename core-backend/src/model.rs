use std::fmt::{self, Display};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub username_or_email: String,
    pub password: String,
    pub extend_session: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FavouriteAnimal {
    Cat,
    Dog,
    Unsure,
}

impl FavouriteAnimal {
    pub fn as_str(&self) -> &str {
        match self {
            FavouriteAnimal::Cat => "cat",
            FavouriteAnimal::Dog => "dog",
            FavouriteAnimal::Unsure => "unsure",
        }
    }
}

impl Display for FavouriteAnimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FavouriteAnimal {
    type Err = InvalidFavouriteAnimal;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Self::Cat),
            "dog" => Ok(Self::Dog),
            "unsure" => Ok(Self::Unsure),
            _ => Err(InvalidFavouriteAnimal(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid favourite animal: {0:?}")]
pub struct InvalidFavouriteAnimal(pub String);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub favourite_animal: FavouriteAnimal,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub email: String,
    pub favourite_animal: FavouriteAnimal,
}

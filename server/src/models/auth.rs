use aragog::Record;
use chrono::{prelude::*, serde::ts_seconds};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Record)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Record, Debug, PartialEq)]
pub struct SessionRecord {
    pub nonce: String,
    #[serde(flatten)]
    pub token: Option<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Token {
    pub token: Option<String>,
    pub name: String,
    pub preferred_username: String,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

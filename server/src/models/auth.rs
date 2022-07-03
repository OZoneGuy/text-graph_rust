use super::generic::Error;
use crate::Result as CResult;
use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use aragog::Record;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Record)]
pub struct User {
    first_name: String,
    last_name: String,
    email: String,
}

#[derive(Serialize, Deserialize, Clone, Record, Debug, PartialEq)]
pub struct SessionRecord {
    pub verifier: String,
    pub token: Option<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: String,
    pub token: String,
    pub creation_date: std::time::SystemTime,
    pub expiration: Option<core::time::Duration>,
    pub refresh_token: Option<String>,
}

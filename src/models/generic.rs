use rocket::serde::Serialize;
use std::fmt::Debug;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Error {
    message: String,
    version: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Health {
    message: String,
    version: String,
}

impl Health {
    pub fn new(message: String) -> Health {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Health { message, version }
    }
}

impl Error {
    pub fn new<E: Debug>(error: E) -> Error {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Error {
            message: format!("{:?}", error),
            version,
        }
    }
}

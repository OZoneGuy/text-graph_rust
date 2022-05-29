use actix_web::{
    body::BoxBody, http::StatusCode, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    ResponseError,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Debug, PartialEq)]
pub struct Error {
    message: String,
    version: String,
    #[serde(skip_serializing)]
    code: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    pub fn new<E: Debug>(e: E, code: StatusCode) -> Error {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Error {
            message: format!("{:?}", e),
            version,
            code: code.as_u16(),
        }
    }

    pub fn default<E: Debug>(e: E) -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Error {
            message: format!("{:?}", e),
            version,
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        }
    }
}

impl Responder for Error {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<BoxBody> {
        let code = StatusCode::from_u16(self.code).expect("Invalid response code");
        HttpResponseBuilder::new(code).json(self)
    }
}

impl Responder for Health {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::Ok().json(self)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).expect("Should have a valid status code")
    }
}

impl std::error::Error for Error {}

use actix_web::{
    body::BoxBody, http::StatusCode, HttpRequest, HttpResponse, HttpResponseBuilder, Responder,
    ResponseError,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Error {
    message: String,
    version: String,
    #[serde(skip_serializing)]
    code: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Health {
    status: Vec<HealthStatus>,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HealthStatus {
    component: String,
    status: String,
    #[serde(skip)]
    healthy: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Generic {
    message: String,
    version: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "Pagination::default_page")]
    pub page: u32,
    #[serde(default = "Pagination::default_size")]
    pub size: u32,
}

impl Pagination {
    fn default_page() -> u32 {
        1
    }
    fn default_size() -> u32 {
        50
    }
}

impl Health {
    pub fn new(status: Vec<HealthStatus>) -> Health {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Health { status, version }
    }
}

impl Responder for Health {
    type Body = actix_web::body::BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::Ok().json(self)
    }
}

impl ResponseError for Health {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

impl core::fmt::Display for Health {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Server unhealthy: {:?}", self.status)
    }
}

impl HealthStatus {
    pub fn new(component: String) -> Self {
        HealthStatus {
            component,
            status: "Healthy".to_string(),
            healthy: true,
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status
    }

    pub fn set_unhealthy(&mut self) {
        self.healthy = false
    }

    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}

impl Generic {
    pub fn new(message: String) -> Generic {
        let version = env!("CARGO_PKG_VERSION").to_string();
        Generic { message, version }
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
impl Responder for Generic {
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

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).json(self)
    }
}

impl std::error::Error for Error {}

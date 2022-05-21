use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct QRefParams {
    pub chapter: i64,
    pub init_verse: i64,
    pub final_verse: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HRefParams {
    pub collection: String,
    pub number: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BRefParams {
    pub isbn: String,
    pub name: String,
    pub page: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RefEnum {
    QRef(QRefParams),
    HRef(HRefParams),
    BRef(BRefParams),
}
impl Responder for RefEnum {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<BoxBody> {
        HttpResponse::Ok().json(self)
    }
}

impl RefEnum {
    pub fn is_book(&self) -> bool {
        match self {
            RefEnum::BRef(_) => true,
            _ => false,
        }
    }
}

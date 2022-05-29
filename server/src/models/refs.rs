use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use arangoq::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, ArangoBuilder)]
pub struct QRef {
    pub chapter: i64,
    pub init_verse: i64,
    pub final_verse: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, ArangoBuilder)]
pub struct HRef {
    pub collection: String,
    pub number: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, ArangoBuilder)]
pub struct BRef {
    pub isbn: String,
    pub name: String,
    pub page: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RefEnum {
    Q(QRef),
    H(HRef),
    B(BRef),
}
impl Responder for RefEnum {
    type Body = BoxBody;
    fn respond_to(self, _: &HttpRequest) -> HttpResponse<BoxBody> {
        HttpResponse::Ok().json(self)
    }
}

impl RefEnum {
    pub fn is_book(&self) -> bool {
        matches!(self, RefEnum::B(_))
    }
}

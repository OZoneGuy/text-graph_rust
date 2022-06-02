use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use aragog::Record;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Record)]
pub struct QRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    pub chapter: i64,
    pub init_verse: i64,
    pub final_verse: i64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Record)]
pub struct HRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    pub collection: String,
    pub number: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    pub isbn: String,
    pub name: String,
    pub page: i64,
}

#[derive(Serialize, Deserialize, Clone, Record)]
pub struct RefEdge {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)] // Removes the tags when serialising and deserialising
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

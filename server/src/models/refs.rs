use super::generic::Error;
use crate::Result as CResult;
use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use aragog::Record;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Record)]
pub struct QRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<String>,
    pub chapter: usize,
    pub init_verse: usize,
    pub final_verse: usize,
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

impl QRef {
    pub fn validate(&self) -> CResult<()> {
        if self.chapter > 114 {
            return Err(Error::default("Invalid chapter number"));
        }
        if self.final_verse < self.init_verse {
            return Err(Error::default("Final verse is before initial verse"));
        }
        Ok(())
    }
}

use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct QRefParams {
    pub chapter: i64,
    pub init_verse: i64,
    pub final_verse: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct HRefParams {
    pub collection: String,
    pub number: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct BRefParams {
    pub isbn: String,
    pub name: String,
    pub page: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub enum RefEnum {
    QRef(QRefParams),
    HRef(HRefParams),
    BRef(BRefParams),
}

use arangoq::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewTopic {
    pub name: String,
    pub id: Option<String>,
}

#[derive(ArangoBuilder, Serialize)]
pub struct Topic {
    name: String,
}

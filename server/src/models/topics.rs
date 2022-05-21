use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewTopic {
    pub name: String,
    pub id: Option<String>,
}

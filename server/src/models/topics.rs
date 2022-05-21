use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct NewTopic {
    pub name: String,
    pub id: Option<String>,
}

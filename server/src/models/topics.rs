use aragog::Record;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq, Record)]
pub struct Topic {
    #[serde(rename = "_key")]
    pub key: Option<String>,
    pub name: String,
}

impl Topic {
    pub fn new(name: &str) -> Self {
        Topic {
            key: name.to_string().into(),
            name: name.to_string(),
        }
    }
}

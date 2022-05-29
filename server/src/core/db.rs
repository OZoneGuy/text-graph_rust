use actix_web::http::StatusCode;
use arangoq::ArangoConnection;
use reqwest::Client;
use std::process::Command;

use crate::models::generic::Error;
use crate::models::refs::*;
use mockall::automock;

type Result<T> = std::result::Result<T, Error>;

pub struct Database {
    host: String,
    db: ArangoConnection,
}

// Locally, I can only have one table/database
// This is made to differentiate between test data and "prod" data
// Any labels prefixed with "Test" is test data.
// Maybe there is a better way to do it?
const TOPIC_LABEL: &str = "Topic";
const QREF_LABEL: &str = "QRef";
const HREF_LABEL: &str = "HRef";
const BREF_LABEL: &str = "BRef";

const REF_RELATION: &str = "REF";

#[automock]
impl Database {
    pub async fn new(cfg: Config) -> Self {
        let c = Client::new();
        Database {
            host: cfg.address.clone(),
            db: ArangoConnection::new(cfg.address, "DB".to_string(), c),
        }
    }

    pub async fn health(&self) -> Result<()> {
        let resp = reqwest::get(&self.host)
            .await
            .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let code = resp.status();
            let text = resp.text().await;
            Err(Error::new(text, code))
        }
    }

    pub fn migrate() -> Result<()> {
        Command::new("../bin/aragog")
            .args(["-u", "root", "migrate"])
            .output()
            .map(|_| ())
            .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR))
    }

    pub async fn get_topics(&self, page: i64, size: i64) -> Result<Vec<String>> {
        let skip = (page - 1) * size;
        todo!()
    }

    pub async fn add_topic(&self, topic: &str) -> Result<()> {
        todo!()
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<()> {
        todo!()
    }

    pub async fn add_qref_to_topic(&self, topic: &str, q_ref: QRef) -> Result<()> {
        todo!()
    }

    pub async fn add_href_to_topic(&self, topic: &str, h_ref: HRef) -> Result<()> {
        todo!()
    }

    pub async fn get_refs(&self, topic: &str) -> Result<Vec<RefEnum>> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub address: String,
    pub port: String,
    pub username: String,
    pub pass: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: "localhost".to_string(),
            port: "7687".to_string(),
            username: "admin".to_string(),
            pass: "".to_string(),
        }
    }
}

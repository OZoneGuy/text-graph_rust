use arangoq::ArangoConnection;
use reqwest::Client;
use std::process::Command;

use crate::models::generic::Error;
use crate::models::refs::*;
use crate::models::topics::{ TopicArangoBuilder, Topic };
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
const TOPIC_LABEL: &str = "TopicCollection";
const QREF_LABEL: &str = "QRefCollection";
const HREF_LABEL: &str = "HRefCollection";
const BREF_LABEL: &str = "BRefCollection";

const REF_RELATION: &str = "RefEdgeCollection";

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
            .map_err(Error::default)?;
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
            .map_err(Error::default)
    }

    pub async fn get_topics(&self, _page: usize, size: usize) -> Result<Vec<String>> {
        // This is an issue, no nice way to skip
        // let skip = (page - 1) * size;
        let q = TopicArangoBuilder::new(TOPIC_LABEL).read()
            .limit(size).build();
        q.try_exec::<Topic>(&self.db).await
            .map_err(Error::default)
            .map(|r| r.result.into_iter().map(|t| t.name).collect())
    }

    pub async fn add_topic(&self, name: &str) -> Result<()> {
        TopicArangoBuilder::new(TOPIC_LABEL)
            .create(&Topic{name: name.to_string()})
            .build()
            .try_exec::<Topic>(&self.db).await
            .map(|_| ())
            .map_err(Error::default)
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<()> {
        TopicArangoBuilder::new(TOPIC_LABEL)
            .delete()
            .filter()
            .name_eq(&topic.to_string())
            .build()
            .try_exec::<Topic>(&self.db).await
            .map(|_| ())
            .map_err(Error::default)
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

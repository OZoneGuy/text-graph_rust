use aragog::query::{Comparison, Filter, Query, QueryResult};
use aragog::transaction::Transaction;
use aragog::{DatabaseConnection, DatabaseRecord, Record};

use crate::models::auth::*;
use crate::models::generic::Error;
use crate::models::refs::*;
use crate::models::topics::Topic;

use mockall::automock;

use std::process::Command;
type Result<T> = std::result::Result<T, Error>;

pub struct Database {
    db: DatabaseConnection,
}

#[automock]
impl Database {
    pub async fn new(cfg: Config) -> Self {
        let db = DatabaseConnection::builder()
            .with_credentials(&cfg.address, &cfg.db_name, &cfg.username, &cfg.pass)
            .with_schema_path(&cfg.schema_path)
            .build()
            .await
            .expect("Failed to create a database connection...");
        Database { db }
    }

    pub async fn health(&self) -> Result<()> {
        Command::new("../bin/aragog")
            .args(["-u", "root", "describe"])
            .output()
            .map(|_| ())
            .map_err(Error::default)
    }

    pub fn migrate() -> Result<()> {
        Command::new("../bin/aragog")
            .args(["-u", "root", "migrate"])
            .output()
            .map(|_| ())
            .map_err(Error::default)
    }

    pub async fn get_topics(&self, page: u32, size: u32) -> Result<Vec<String>> {
        let skip = (page - 1) * size;
        Topic::query()
            .limit(size, Some(skip))
            .call(&self.db)
            .await
            .map(|r: QueryResult<Topic>| r.0.into_iter().map(|r| r.record.name).collect())
            .map_err(Error::default)
    }

    pub async fn add_topic(&self, name: &str) -> Result<()> {
        let t = Topic::new(name);
        DatabaseRecord::create(t, &self.db)
            .await
            .map_err(Error::default)
            .map(|_| ())
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<()> {
        DatabaseRecord::<Topic>::find(topic, &self.db)
            .await
            .map_err(Error::default)?
            .delete(&self.db)
            .await
            .map(Into::into)
            .map_err(Error::default)
    }

    pub async fn add_qref_to_topic(&self, topic: &str, q_ref: QRef) -> Result<()> {
        let t = Transaction::new(&self.db).await.map_err(Error::default)?;
        t.safe_execute(|con| async move {
            let r = DatabaseRecord::create(q_ref, &con).await?;
            let to = Topic::find(topic, &con).await?;
            DatabaseRecord::link(&to, &r, &con, RefEdge {}).await?;
            log::debug!("linked topic");
            Ok(())
        })
        .await
        .and_then(Into::into)
        .map_err(Error::default)
    }

    pub async fn add_href_to_topic(&self, topic: &str, h_ref: HRef) -> Result<()> {
        let t = Transaction::new(&self.db).await.map_err(Error::default)?;
        t.safe_execute(|con| async move {
            let r = DatabaseRecord::create(h_ref, &con).await?;
            let t = Topic::find(topic, &con).await?;
            DatabaseRecord::link(&t, &r, &con, RefEdge {}).await?;
            Ok(())
        })
        .await
        .and_then(Into::into)
        .map_err(Error::default)
    }

    pub async fn get_refs(&self, topic: &str) -> Result<Vec<RefEnum>> {
        // Find all Refs
        let r = Query::outbound(
            1,
            1,
            RefEdge::COLLECTION_NAME,
            &format!("{}/{}", Topic::COLLECTION_NAME, topic),
        )
        .call(&self.db)
        .await
        .map_err(Error::default)?;

        // Get all QRefs
        let mut q: Vec<RefEnum> = r
            .get_records::<QRef>()
            .iter()
            .map(|r| RefEnum::Q(r.record.clone()))
            .collect();
        // Get all HRefs
        let mut h: Vec<RefEnum> = r
            .get_records::<HRef>()
            .iter()
            .map(|r| RefEnum::H(r.record.clone()))
            .collect();

        // Put them all in one place
        q.append(&mut h);
        Ok(q)
    }

    pub async fn get_qrefs(&self, topic: &str, page: u32, size: u32) -> Result<Vec<QRef>> {
        let skip = (page - 1) * size;
        let r = Query::outbound(
            1,
            1,
            RefEdge::COLLECTION_NAME,
            &format!("{}/{}", Topic::COLLECTION_NAME, topic),
        )
        .limit(size, Some(skip))
        .call(&self.db)
        .await
        .map_err(Error::default)?;
        Ok(r.get_records::<QRef>()
            .iter()
            .map(|r| r.record.clone())
            .collect())
    }

    /// Get the list of topics pointing at this verse.
    /// Finds all the references containing this verse:
    /// ```
    ///  qref.chapter == result.chapter &&
    ///     qref.init_verse >= result.init_verse
    ///     qref.final_verse <= result.final_verse
    /// ```
    /// Then gets all the topic names that are pointing to these references.
    ///
    /// qref: The verse reference, can be multiple verses but must be contiguous.
    /// page: the page to get.
    /// size: The size of each page.
    pub async fn get_topics_from_qref(
        &self,
        qref: QRef,
        page: u32,
        size: u32,
    ) -> Result<Vec<String>> {
        let skip = (page - 1) * size;
        // Create query
        let qrefs_query = QRef::query().filter(
            Filter::new(Comparison::field("chapter").equals(qref.chapter))
                .and(Comparison::field("init_verse").lesser_or_equal(qref.init_verse))
                .and(Comparison::field("final_verse").greater_or_equal(qref.final_verse)),
        );

        let qrefs: Vec<Query> = QRef::get(&qrefs_query, &self.db)
            .await
            .map_err(Error::default)?
            .0
            .iter()
            .map(|r| {
                r.inbound_query(1, 1, RefEdge::COLLECTION_NAME)
                    .limit(size, Some(skip))
            })
            .collect();
        let mut topics: Vec<String> = vec![];

        for ref_query in qrefs {
            let newtopics: Vec<String> = ref_query
                .call(&self.db)
                .await
                .map_err(Error::default)?
                .0
                .iter()
                .map(|r: &DatabaseRecord<Topic>| r.record.name.clone())
                .collect();
            topics.extend(newtopics);
        }
        // remove duplicate topic names
        topics.dedup();
        Ok(topics)
    }

    pub async fn add_session(&self, key: String, session: SessionRecord) -> Result<()> {
        DatabaseRecord::create_with_key(session, key, &self.db)
            .await
            .map_err(Error::default)
            .map(|_| ())
    }

    pub async fn get_session(&self, state: String) -> Result<SessionRecord> {
        SessionRecord::find(&state, &self.db)
            .await
            .map(|r| r.record)
            .map_err(Error::default)
    }

    pub async fn update_session(&self, state: String, token: Token) -> Result<SessionRecord> {
        let mut sess_doc: DatabaseRecord<SessionRecord> = SessionRecord::find(&state, &self.db)
            .await
            .map_err(Error::default)?;
        sess_doc.token = Some(token);
        sess_doc.save(&self.db).await.map_err(Error::default)?;
        Ok(sess_doc.record)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub address: String,
    pub username: String,
    pub pass: String,
    pub db_name: String,
    pub schema_path: String,
}

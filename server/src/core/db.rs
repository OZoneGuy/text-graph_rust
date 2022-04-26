use neo4rs::{query, Graph, Node, Result};

use crate::models::refs::*;

pub struct Database {
    graph_db: Graph,
}

const TOPIC_LABEL: &str = "Topic";
const QREF_LABEL: &str = "QRef";
const HREF_LABEL: &str = "HRef";
const BREF_LABEL: &str = "BRef";

const REF_RELATION: &str = "REF";

impl Database {
    pub async fn new(cfg: Config) -> Self {
        debug!("Attempting connection with config: {:?}", cfg);
        let config = neo4rs::config()
            .uri(format!("{}:{}", cfg.address, cfg.port).as_str())
            .user(cfg.username.as_str())
            .password(cfg.pass.as_str())
            .build()
            .unwrap();
        let new_graph = Graph::connect(config).await;
        match new_graph {
            Ok(g) => {
                if let Err(e) = g.run(query("RETURN 1")).await {
                    error!("Failed to connect to the database: {:#?}", e);
                    panic!("Failed to connect to the database: {:#?}", e);
                }

                info!("Connected to database!");
                Database { graph_db: g }
            }
            Err(e) => {
                error!("Failed to connect to the database: {:#?}", e);
                panic!("Failed to connect to the database: {:#?}", e);
            }
        }
    }

    pub async fn health(&self) -> Result<()> {
        self.graph_db.run(query("RETURN 1")).await
    }

    pub async fn get_topics(&self, page: i64, size: i64) -> Result<Vec<String>> {
        let skip = (page - 1) * size;
        let mut res = self
            .graph_db
            .execute(
                query(
                    format!("MATCH (t:{} ) RETURN t SKIP $skip LIMIT $size", TOPIC_LABEL).as_str(),
                )
                .param("skip", skip)
                .param("size", size),
            )
            .await?;
        let mut topics: Vec<String> = vec![];

        while let Some(row) = res.next().await? {
            if let Some(name) = row.get::<Node>("t").unwrap().get("name") {
                topics.push(name);
            }
        }

        return Ok(topics);
    }

    pub async fn add_topic(&self, topic: &str) -> Result<()> {
        self.graph_db
            .run(
                query(format!("CREATE (t:{} {{name: $name, level: 0}})", TOPIC_LABEL).as_str())
                    .param("name", topic.to_string()),
            )
            .await
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<()> {
        self.graph_db
            .run(
                query(format!("MATCH (t:{} {{name: $name}}) DELETE t", TOPIC_LABEL).as_str())
                    .param("name", topic),
            )
            .await
    }

    pub async fn add_qref_to_topic(&self, topic: &str, q_ref: QRefParams) -> Result<()> {
        let q = format!(
            "MERGE (t:{} {{name: $topic}})-[r:{}]->(qr:{} {{chapter: $chapter, init_verse: $i_verse, final_verse: $f_verse}})",
            TOPIC_LABEL, REF_RELATION, HREF_LABEL);

        self.graph_db
            .run(
                query(q.as_str())
                    .param("topic", topic)
                    .param("chapter", q_ref.chapter)
                    .param("init_verse", q_ref.init_verse)
                    .param("final_verse", q_ref.final_verse),
            )
            .await
    }

    pub async fn add_href_to_topic(&self, topic: &str, id: i64) -> Result<()> {
        let q = format!(
            "MERGE (t:{0} {{name: $topic}})-[r:{1}]->(qr:{2} {{h_id: $id}})",
            TOPIC_LABEL, REF_RELATION, QREF_LABEL
        );

        self.graph_db
            .run(query(q.as_str()).param("h_id", id).param("topic", topic))
            .await
    }

    pub async fn add_bref_to_topic(&self, topic: &str, bref: BRefParams) -> Result<()> {
        let q = format!(
            "MERGE (t:{0} {{name: $topic}})-[r:{1}]->(qr:{2} {{ isbn: $isbn, name: $name, page: $page }})",
            TOPIC_LABEL, REF_RELATION, BREF_LABEL);

        self.graph_db
            .run(
                query(q.as_str())
                    .param("topic", topic)
                    .param("isbn", bref.isbn)
                    .param("name", bref.name)
                    .param("page", bref.page),
            )
            .await
    }

    pub async fn get_refs(&self, topic: &str) -> Result<Vec<RefEnum>> {
        let q = format!(
            "MATCH (:{0} {{name: $topic}})-[:{1}]->(r)",
            TOPIC_LABEL, REF_RELATION
        );

        let mut res = self
            .graph_db
            .execute(query(q.as_str()).param("topic", topic))
            .await?;

        let mut refs: Vec<RefEnum> = vec![];

        while let Some(row) = res.next().await? {
            let node = row
                .get::<Node>("t")
                .expect("Row should have an element 't'.");
            let labels = node.labels();
            if labels.contains(&QREF_LABEL.to_string()) {
                let q_ref = QRefParams {
                    chapter: node
                        .get("chapter")
                        .expect("Couldn't find chapter attribute in QRef node."),
                    init_verse: node
                        .get("init_verse")
                        .expect("Couldn't find init_verse attribute in QRef node."),
                    final_verse: node
                        .get("final_verse")
                        .expect("Couldn't find final_verse attribute in QRef node."),
                };
                refs.push(RefEnum::QRef(q_ref));
            } else if labels.contains(&HREF_LABEL.to_string()) {
                refs.push(RefEnum::HRef(
                    node.get("h_id")
                        .expect("Couldn't find 'h_id' attribute in HRef node."),
                ));
            } else if labels.contains(&BREF_LABEL.to_string()) {
                let b_ref = BRefParams {
                    isbn: node
                        .get("isbn")
                        .expect("Couldn't find isbn attribute in BRef node."),
                    name: node
                        .get("name")
                        .expect("Couldn't find name attribute in BRef node."),
                    page: node
                        .get("page")
                        .expect("Couldn't find page attribute in BRef node."),
                };
                refs.push(RefEnum::BRef(b_ref));
            }
        }

        Ok(refs)
    }
}

#[derive(Debug)]
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

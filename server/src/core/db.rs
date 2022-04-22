use neo4rs::{query, Graph, Node, Result, Error};

pub struct Database {
    graph_db: Graph,
}

const TOPIC_LABEL: &str = "Topic";
const SUPER_TOPIC_LABEL: &str = "SUPER_TOPIC";

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
                query("MATCH (t:$label ) RETURN t SKIP $skip LIMIT $size")
                    .param("label", TOPIC_LABEL)
                    .param("skip", skip)
                    .param("size", size),
            )
            .await?;
        let mut topics: Vec<String> = vec![];

        while let Ok(Some(row)) = res.next().await {
            if let Some(name) = row.get::<Node>("t").unwrap().get("name") {
                topics.push(name);
            }
        }

        return Ok(topics);
    }

    pub async fn add_topic(&self, topic: &str) -> Result<()> {
        self.graph_db
            .run(
                query("CREATE (t:$label {name: $name})")
                    .param("name", topic.to_string())
                    .param("label", TOPIC_LABEL),
            )
            .await
    }

    pub async fn delete_topic(&self, topic: &str) -> Result<()> {
        self.graph_db
            .run(
                query("MATCH (t:$label {name: $name}) DELETE t")
                    .param("name", topic)
                    .param("label", TOPIC_LABEL),
            )
            .await
    }

    pub async fn sub_topic_relation(&self, topic: &str, sub_topic: &str) -> Result<()> {
        if topic == sub_topic {
            return Err(Error::UnknownMessage("Topic cannot also be sub topic.".to_string()));
        }
        self.graph_db
            .execute(
                query(
                    "MATCH (t:$label {name: $topic}), (st:$label {name: $sub_topic})
                     MERGE (t)-[r:rel_label]->(st) RETURN r",
                )
                .param("topic", topic)
                .param("sub_topic", sub_topic)
                .param("label", TOPIC_LABEL)
                .param("rel_label", SUPER_TOPIC_LABEL),
            )
            .await?
            .next()
            .await
            .map(|_| ())
    }

    pub async fn get_sub_topics(&self, topic: &str) -> Result<Vec<String>> {
        let mut res = self
            .graph_db
            .execute(
                query("MATCH (:Topic {name: $name})-[:SUPER_TOPIC]->(t:Topic) RETURN t")
                    .param("name", topic),
            )
            .await?;
        let mut topics: Vec<String> = vec![];
        while let Some(row) = res.next().await? {
            if let Some(name) = row.get::<Node>("t").unwrap().get("name") {
                topics.push(name);
            }
        }
        Ok(topics)
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

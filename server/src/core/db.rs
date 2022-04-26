use neo4rs::{query, Graph, Node, Result};

pub struct Database {
    graph_db: Graph,
}

const TOPIC_LABEL: &str = "Topic";

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

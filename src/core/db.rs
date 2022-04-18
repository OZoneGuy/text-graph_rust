use log::{debug, error, info};
use neo4rs::*;

pub struct Database {
    graph_db: Graph,
}

impl Database {
    pub async fn new(cfg: Config) -> Self {
        debug!("Attempting connection with config: {:?}", cfg);
        let new_graph = Graph::new(
            format!("{}:{}", cfg.address, cfg.port).as_str(),
            cfg.username.as_str(),
            cfg.pass.as_str(),
        )
        .await;
        match new_graph {
            Ok(g) => {
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

    pub async fn get_topics(&self) -> Result<Vec<String>> {
        let res = self
            .graph_db
            .execute(query("MATCH (t: Topic) RETURN t"))
            .await;
        match res {
            Ok(mut r) => {
                let mut topics: Vec<String> = vec![];

                while let Ok(Some(row)) = r.next().await {
                    if let Some(name) = row.get::<Node>("t").unwrap().get("name") {
                        topics.push(name);
                    }
                }

                return Ok(topics);
            }
            Err(e) => Err(e),
        }
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

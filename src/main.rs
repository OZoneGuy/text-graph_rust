#[macro_use]
extern crate rocket;
use log::info;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;

#[path = "models/generic.rs"]
mod generic;
use generic::*;
#[path = "models/topics.rs"]
mod topics;
use topics::*;
#[path = "core/db.rs"]
mod db;
use db::*;

#[launch]
async fn rocker() -> _ {
    let cfg = db::Config {
        ..Default::default()
    };
    rocket::build()
        .manage(Database::new(cfg).await)
        .mount("/", routes![health, root, get_topics, add_topic])
}

#[get("/healthz")]
async fn health(db: &State<Database>) -> Json<Health> {
    let mut health_check: HashMap<&str, String> = HashMap::new();

    if let Some(db_err) = db.health().await.err() {
        health_check.insert("Database", format!("{:?}", db_err ));
    }

    if health_check.len() != 0 {
        let mut health_string: String = "Not healthy :(:".to_string();

        for (service, err) in &health_check {
            health_string.push_str(format!("\tService: {}, Error: {}", service, err).as_str())
        }

        Json(Health::new(health_string))
    } else {
        Json(Health::new("Everything is fine...".to_string()))
    }
}

#[get("/")]
fn root() -> String {
    "Nothing to see here!".to_string()
}

#[get("/topics")]
async fn get_topics(db: &State<Database>) -> Result<Json<Vec<String>>, Json<Error>> {
    db.get_topics()
        .await
        .map(|v| Json(v))
        .map_err(|e| Json(Error::new(format!("Database error: {:?}", e))))
}

#[post("/topics", format = "json", data = "<topic>")]
async fn add_topic(
    topic: Json<NewTopic>,
    db: &State<Database>,
) -> Result<Json<Health>, Json<Error>> {
    info!(target: "app_events", "New topic json: {:#?}", topic.0);

    db.add_topic(topic.name.as_str())
        .await
        .map(|_| Json(Health::new(format!("Successfully created {}", topic.name))))
        .map_err(|e| Json(Error::new(e)))
}

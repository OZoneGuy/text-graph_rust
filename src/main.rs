#[macro_use]
extern crate rocket;
use log::info;
use rocket::serde::json::Json;
use rocket::State;

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
fn rocker() -> _ {
    let cfg = db::Config {
        ..Default::default()
    };
    rocket::build()
        .mount("/", routes![health, root, get_topics, add_topic])
        .manage(Database::new(cfg))
}

#[get("/healthz")]
fn health() -> Json<Health> {
    Json(Health::new("Everything is fine...".to_string()))
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
        .map_err(|e| Json(Error::new(e)))
}

#[post("/topics", format = "json", data = "<topic>")]
fn add_topic(topic: Json<NewTopic>) -> Result<Json<Health>, Json<Error>> {
    info!(target: "app_events", "New topic json: {:#?}", topic.0);

    Ok(Json(Health::new(format!(
        "Successfully added: {:?}",
        topic.0
    ))))
}

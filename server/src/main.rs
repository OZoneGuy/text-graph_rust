#[macro_use]
extern crate rocket;
use clap::{crate_authors, crate_name, crate_version, Arg, ArgGroup, Command};
use rocket::{serde::json::Json, State};
use std::collections::HashMap;

mod models;
use models::{generic::*, refs::*, topics::*};

mod core;
use crate::core::{auth::*, db::*};

type TextResponse = Result<Json<Health>, Json<Error>>;

fn app<'help>() -> Command<'help> {
    Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .arg(Arg::new("dev")
             .short('d')
             .long("dev")
             .help("Enable dev mode.")
             .long_help("Enable more logging, sends more verbose errors to the client, and uses local database."))
        .arg(Arg::new("db_username")
             .short('u')
             .help("The username for the Graph database. Defaults to neo4j")
             .env("DB_USERNAME")
             .takes_value(true)
             .default_value("neo4j"))
        .arg(Arg::new("db_pass")
             .short('p')
             .help("The password for the Graph database.")
             .env("DB_PASSWORD")
             .takes_value(true)
             .required(true))
        .arg(Arg::new("db_port")
             .short('P')
             .help("The port for the Graph database.")
             .env("DB_PORT")
             .takes_value(true)
             .default_value("7687"))
        .arg(Arg::new("db_host")
             .short('H')
             .help("The host for the Graph database.")
             .env("DB_HOST")
             .takes_value(true)
             .default_value("localhost"))
        .group(ArgGroup::new("database")
               .args(&["db_username", "db_pass", "db_port", "db_host"]))
}

#[launch]
async fn rocket() -> _ {
    let args = app().get_matches();

    let log_level: rocket::config::LogLevel;
    if args.is_present("dev") {
        log_level = rocket::config::LogLevel::Debug;
    } else {
        log_level = rocket::config::LogLevel::Normal
    }

    let cfg = core::db::Config {
        username: args
            .value_of("db_username")
            .expect("Empty username value.")
            .to_string(),
        pass: args
            .value_of("db_pass")
            .expect("Empty password value")
            .to_string(),
        port: args
            .value_of("db_port")
            .expect("Empty port value")
            .to_string(),
        address: args
            .value_of("db_host")
            .expect("Empty host value")
            .to_string(),
    };
    rocket::build()
        .manage(Database::new(cfg).await)
        .mount(
            "/",
            routes![
                health,
                root,
                get_topics,
                add_topic,
                login,
                delete_topic,
                get_references,
                add_qref,
                add_href,
                // add_bref,
            ],
        )
        .configure(rocket::Config {
            log_level,
            ..Default::default()
        })
}

#[get("/healthz")]
async fn health(db: &State<Database>) -> Json<Health> {
    let mut health_check: HashMap<&str, String> = HashMap::new();

    if let Some(db_err) = db.health().await.err() {
        health_check.insert("Database", format!("{:?}", db_err));
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

#[get("/topics?<page>&<size>")]
async fn get_topics(
    page: Option<i64>,
    size: Option<i64>,
    db: &State<Database>,
) -> Result<Json<Vec<String>>, Json<Error>> {
    let page_num = page.unwrap_or(1);
    let size_num = size.unwrap_or(50);
    db.get_topics(page_num, size_num)
        .await
        .map(|v| Json(v))
        .map_err(|e| Json(Error::new(format!("Database error: {:?}", e))))
}

#[post("/topics", format = "json", data = "<topic>")]
async fn add_topic(
    _auth: AuthHandler,
    topic: Json<NewTopic>,
    db: &State<Database>,
) -> TextResponse {
    info!("New topic json: {:#?}", topic.0);

    db.add_topic(topic.name.as_str())
        .await
        .map(|_| Json(Health::new(format!("Successfully created {}", topic.name))))
        .map_err(|e| Json(Error::new(e)))
}

#[delete("/topics", format = "json", data = "<topic>")]
async fn delete_topic(topic: Json<NewTopic>, db: &State<Database>) -> TextResponse {
    db.delete_topic(topic.name.as_str())
        .await
        .map(|_| {
            info!("Deleted {}", topic.name);
            Json(Health::new(format!("Successfully delete {}", topic.name)))
        })
        .map_err(|e| {
            error!("Failed to delete topic: {:?}", e);
            Json(Error::new(format!("Failed to delete topic: {:?}", e)))
        })
}

#[get("/login")]
fn login(_auth: AuthHandler) -> Json<Health> {
    Json(Health::new("You are now logged in!".to_string()))
}

#[get("/refs?<topic>")]
async fn get_references(
    topic: &str,
    db: &State<Database>,
) -> Result<Json<Vec<RefEnum>>, Json<Error>> {
    db.get_refs(topic)
        .await
        .map(|v| Json(v.into_iter().filter(|r| !r.isBook()).collect()))
        .map_err(|e| Json(Error::new(e)))
}

#[post("/refs/qref?<topic>", format = "json", data = "<qref>")]
async fn add_qref(
    topic: &str,
    qref: Json<QRefParams>,
    db: &State<Database>,
) -> Result<Json<Health>, Json<Error>> {
    db.add_qref_to_topic(topic, qref.0)
        .await
        .map(|_| Json(Health::new("".to_string())))
        .map_err(|e| Json(Error::new(e)))
}

#[post("/refs/href?<topic>", format = "json", data = "<href>")]
async fn add_href(
    topic: &str,
    href: Json<HRefParams>,
    db: &State<Database>,
) -> Result<Json<Health>, Json<Error>> {
    db.add_href_to_topic(topic, href.0)
        .await
        .map(|_| Json(Health::new("".to_string())))
        .map_err(|e| Json(Error::new(e)))
}

#[post("/refs/bref?<topic>", format = "json", data = "<bref>")]
async fn add_bref(
    topic: &str,
    bref: Json<BRefParams>,
    db: &State<Database>,
) -> Result<Json<Health>, Json<Error>> {
    db.add_bref_to_topic(topic, bref.0)
        .await
        .map(|_| Json(Health::new("".to_string())))
        .map_err(|e| Json(Error::new(e)))
}

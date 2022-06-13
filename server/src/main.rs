#![cfg_attr(test, allow(dead_code))]
mod core;
mod http;
mod models;

use crate::core::db::{Config, Database};
use http::{refs::refs_service, root::root_service, topics::topics_service};
use models::generic::Error;

use actix_web::{web, App, HttpServer};
use clap::{crate_authors, crate_name, crate_version, Arg, ArgGroup, Command};

type Result<T> = std::result::Result<T, Error>;

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
             .default_value("user"))
        .arg(Arg::new("db_pass")
             .short('p')
             .help("The password for the Graph database.")
             .env("DB_PASSWORD")
             .takes_value(true)
             .required(true))
        .arg(Arg::new("db_host")
             .short('H')
             .help("The host for the Graph database.")
             .env("DB_HOST")
             .takes_value(true)
             .default_value("localhost"))
        .arg(Arg::new("db_name")
             .short('N')
             .help("The name of the database")
             .env("DB_NAME")
             .default_value("DB"))
        .arg(Arg::new("schema")
             .short('S')
             .help("The path of the schema")
             .env("SCHEMA_PATH")
             .takes_value(true))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = app().get_matches();

    let db_cfg = Config {
        username: args
            .value_of("db_username")
            .expect("Empty username value.")
            .to_string(),
        pass: args
            .value_of("db_pass")
            .expect("Empty password value")
            .to_string(),
        address: args
            .value_of("db_host")
            .expect("Empty host value")
            .to_string(),
        db_name: args
            .value_of("db_name")
            .expect("Empty database value.")
            .to_string(),
        schema_path: args
            .value_of("schema")
            .expect("Empty schema path.")
            .to_string(),
    };

    Database::migrate().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Database::new(db_cfg.clone())))
            .service(
                web::scope("/api/v1/")
                    .configure(root_service)
                    .configure(topics_service)
                    .configure(refs_service),
            )
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

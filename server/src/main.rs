mod core;
mod http;
mod models;
#[cfg(test)]
mod test;

use crate::core::db::{Config, Database};
use http::{root::root_service, topics::topics_service, refs::refs_service};

// use crate::core::auth::*;

use actix_web::{web, App, HttpServer};
use clap::{crate_authors, crate_name, crate_version, Arg, ArgGroup, Command};

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
        port: args
            .value_of("db_port")
            .expect("Empty port value")
            .to_string(),
        address: args
            .value_of("db_host")
            .expect("Empty host value")
            .to_string(),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Database::new(db_cfg.clone())))
            .service(
                web::scope("/api/v1/")
                    .configure(root_service)
                    .configure(topics_service)
                    .configure(refs_service)
            )
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}

// #[get("/login")]
// fn login(_auth: AuthHandler) -> Json<Health> {
//     Json(Health::new("You are now logged in!".to_string()))
// }

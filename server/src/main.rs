#![cfg_attr(test, allow(dead_code))]
mod core;
mod http;
mod models;

use crate::core::auth::AuthHandler;
use crate::core::db::{Config, Database};
use crate::http::{
    auth::auth_service, refs::refs_service, root::root_service, topics::topics_service,
};
use models::generic::Error;

use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;

#[cfg(debug_assertions)]
use dotenv::{dotenv, from_filename};

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[clap(author, version)]
struct Args {
    /// Enable debugging
    #[clap(long, short, action = clap::ArgAction::SetTrue)]
    dev: bool,
    /// The database username
    #[clap(long, value_parser, env = "DB_USERNAME")]
    db_username: String,
    /// The database password
    #[clap(long, value_parser, env = "DB_PASSWORD")]
    db_pass: String,
    /// The database name
    #[clap(long, value_parser, env = "DB_NAME")]
    db_name: String,
    /// The hostname to the database
    #[clap(long, value_parser, env = "DB_HOST")]
    db_host: String,
    /// Path to the schema.
    #[clap(short, long, value_parser, env = "SCHEMA_PATH")]
    schema_path: String,
    /// Azure AD Client Secret
    #[clap(long, value_parser, env = "CLIENT_SECRET")]
    client_secret: String,
    /// Azure AD Client ID
    #[clap(long, value_parser, env = "CLIENT_ID")]
    client_id: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    {
        dotenv().ok();
        from_filename(".secrets.env").ok();
    }

    let args = Args::parse();

    if args.dev {
        env_logger::Builder::new().filter_level(log::LevelFilter::Debug);
    }

    let db_cfg = Config {
        username: args.db_username,
        pass: args.db_pass,
        address: args.db_host,
        db_name: args.db_name,
        schema_path: format!("{}/schema.yaml", args.schema_path),
    };

    Database::migrate().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let db_fact = move || {
        let cfg = db_cfg.clone();
        async move {
            let db = Database::new(cfg).await;
            Ok::<Database, ()>(db)
        }
    };

    let client_secret: String = args.client_secret;
    let client_id: String = args.client_id;

    println!("Running the server...");
    HttpServer::new(move || {
        let policy = actix_identity::CookieIdentityPolicy::new(&[0; 32])
            .name("ir_session")
            .secure(true);

        App::new()
            .wrap(Logger::default())
            .data_factory(db_fact.clone())
            .app_data(actix_web::web::Data::new(AuthHandler::new(
                "http://localhost:8000".to_string(),
                client_secret.clone(),
                client_id.clone(),
            )))
            .wrap(actix_identity::IdentityService::new(policy))
            .service(
                web::scope("/api/v1")
                    .configure(topics_service)
                    .configure(refs_service)
                    .configure(auth_service)
                    .configure(root_service),
            )
    })
    .bind(("localhost", 8000))?
    .run()
    .await
}

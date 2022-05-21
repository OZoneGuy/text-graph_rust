use actix_web::web::{scope, Data, ServiceConfig};
use actix_web::{get, services, Responder};
use std::collections::HashMap;

use mockall_double::double;

#[double]
use crate::core::db::Database;
use crate::models::generic::Health;

pub fn root_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/").service(services![health, root]));
}

#[get("/healthz")]
async fn health(db: Data<Database>) -> Health {
    let mut health_check: HashMap<&str, String> = HashMap::new();

    if let Some(db_err) = db.health().await.err() {
        health_check.insert("Database", format!("{:?}", db_err));
    }

    if health_check.len() != 0 {
        Health::new(hash_to_health(health_check))
    } else {
        Health::new("Everything is fine...".to_string())
    }
}

fn hash_to_health(h: HashMap<&str, String>) -> String {
    let mut health_string: String = "Not healthy :(:\n".to_string();

    for (service, err) in &h {
        health_string.push_str(format!("\tService: {}, Error: {}", service, err).as_str())
    }
    health_string
}

#[get("/")]
async fn root() -> impl Responder {
    "Nothing to see here!"
}

#[cfg(test)]
mod test {
    use super::*;
    // use crate::core::db::Config;
    use actix_service::Service;
    use actix_web::{http::StatusCode, test, web::Bytes, App,
    test::{ read_body, read_body_json, init_service, TestRequest }};

    // fn db_cfg() -> Config {
    //     Config {
    //         ..Default::default()
    //     }
    // }

    #[test]
    async fn test_root() {
        let app = init_service(App::new().service(root)).await;
        let req = TestRequest::with_uri("/").to_request();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body, Bytes::from_static(b"Nothing to see here!"))
    }

    #[test]
    async fn test_health() {
        let mut db = Database::default();
        db.expect_health().returning(|| Ok(()));
        let app = test::init_service(
            App::new()
                .service(health)
                .app_data(Data::new(db)),
        )
        .await;
        let req = test::TestRequest::with_uri("/healthz").to_request();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Health = read_body_json(resp).await;
        assert_eq!(body, Health::new("Everything is fine...".to_string()))
    }
}

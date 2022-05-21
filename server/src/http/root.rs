use actix_web::web::{scope, Data, Json, ServiceConfig};
use actix_web::{get, services, Responder};
use std::collections::HashMap;

use crate::core::db::Database;
use crate::models::generic::Health;

pub fn root_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/").service(services![health, root]));
}

#[get("/healthz")]
async fn health(db: Data<Database>) -> Json<Health> {
    let mut health_check: HashMap<&str, String> = HashMap::new();

    if let Some(db_err) = db.health().await.err() {
        health_check.insert("Database", format!("{:?}", db_err));
    }

    if health_check.len() != 0 {
        let mut health_string: String = "Not healthy :(:\n".to_string();

        for (service, err) in &health_check {
            health_string.push_str(format!("\tService: {}, Error: {}", service, err).as_str())
        }

        Json(Health::new(health_string))
    } else {
        Json(Health::new("Everything is fine...".to_string()))
    }
}

#[get("/")]
async fn root() -> impl Responder {
    "Nothing to see here!"
}

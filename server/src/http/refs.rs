use actix_web::web::{scope, Data, Json, Query, ServiceConfig};
use actix_web::{get, http::StatusCode, post, services, Result};

use crate::core::db::Database;
use crate::models::generic::*;
use crate::models::refs::*;

pub fn refs_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/refs").service(services![get_references, add_qref]));
}

#[get("/")]
async fn get_references(topic: Query<String>, db: Data<Database>) -> Result<Json<Vec<RefEnum>>> {
    db.get_refs(topic.as_str())
        .await
        .map(|v| Json(v.into_iter().filter(|r| !r.is_book()).collect()))
        .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
}

#[post("/qref")]
async fn add_qref(
    topic: Query<String>,
    qref: Json<QRef>,
    db: Data<Database>,
) -> Result<Health> {
    db.add_qref_to_topic(topic.as_str(), qref.0)
        .await
        .map(|_| Health::new("".to_string()))
        .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
}

#[post("/refs/href")]
async fn add_href(
    topic: Query<String>,
    href: Json<HRef>,
    db: Data<Database>,
) -> Result<Health> {
    db.add_href_to_topic(topic.as_str(), href.0)
        .await
        .map(|_| Health::new("".to_string()))
        .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
}

use actix_web::web::{scope, Data, Json, Query, ServiceConfig};
use actix_web::{delete, get, http::StatusCode, post, services, Result};
use serde::Deserialize;

use crate::core::db::Database;
use crate::models::generic::*;
use crate::models::topics::*;

#[derive(Deserialize)]
struct Pagination {
    page: Option<i64>,
    size: Option<i64>,
}

pub fn topics_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/topics").service(services![get_topics, add_topic]));
}

#[get("/")]
async fn get_topics(db: Data<Database>, q: Query<Pagination>) -> Result<Json<Vec<String>>, Error> {
    const DEF_PAGE: i64 = 1;
    const DEF_SIZE: i64 = 50;
    let page_num = q.page.unwrap_or(DEF_PAGE);
    let size_num = q.size.unwrap_or(DEF_SIZE);
    if page_num <= 0 || size_num <= 0 {
        return Err(Error::new(
            "Invalid query paramaters. Must be positive integers.".to_string(),
            StatusCode::BAD_REQUEST,
        ));
    };
    db.get_topics(page_num, size_num)
        .await
        .map(|v| Json(v))
        .map_err(|e| {
            Error::new(
                format!("Database error: {:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })
}

#[post("/")]
async fn add_topic(topic: Json<NewTopic>, db: Data<Database>) -> Result<Health> {
    db.add_topic(topic.name.as_str())
        .await
        .map(|_| Health::new(format!("Successfully created {}", topic.name)))
        .map_err(|e| Error::new(e, StatusCode::INTERNAL_SERVER_ERROR).into())
}

#[delete("/")]
async fn delete_topic(topic: Json<NewTopic>, db: Data<Database>) -> Result<Health> {
    db.delete_topic(topic.name.as_str())
        .await
        .map(|_| {
            // info!("Deleted {}", topic.name);
            Health::new(format!("Successfully delete {}", topic.name))
        })
        .map_err(|e| {
            // error!("Failed to delete topic: {:?}", e);
            Error::new(
                format!("Failed to delete topic: {:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into()
        })
}

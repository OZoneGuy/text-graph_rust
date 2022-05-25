use actix_web::web::{scope, Data, Json, Query, ServiceConfig};
use actix_web::{delete, get, http::StatusCode, post, services, Result};
use serde::Deserialize;

#[mockall_double::double]
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
        .map(Json)
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
            Health::new(format!("Successfully deleted {}", topic.name))
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

#[cfg(test)]
mod test {
    use super::*;
    use actix_service::Service;
    use actix_web::{
        http::StatusCode,
        test,
        test::{init_service, read_body, read_body_json, TestRequest},
        App,
    };
    use neo4rs::unexpected;

    #[test]
    async fn test_get_topics() {
        let mut db = Database::default();
        db.expect_get_topics()
            .returning(|_page, _size| Ok(vec!["topic1".to_string(), "topic2".to_string()]));

        let app = init_service(App::new().service(get_topics).app_data(Data::new(db))).await;
        let req = TestRequest::with_uri("/").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<String> = read_body_json(resp).await;
        assert_eq!(body, vec!["topic1".to_string(), "topic2".to_string()]);
    }

    #[test]
    async fn test_get_topics_bad_query() {
        let db = Database::default();
        let app = init_service(App::new().service(get_topics).app_data(Data::new(db))).await;
        let req = TestRequest::with_uri("/?page=-1&size=-1").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_body(resp).await;
        assert_eq!(
            body,
            format!(
                "{}",
                actix_web::Error::from(Error::new(
                    "Invalid query paramaters. Must be positive integers.".to_string(),
                    StatusCode::from_u16(400).unwrap()
                ))
            )
        )
    }

    #[test]
    async fn test_get_topics_partial_query() {
        let mut db = Database::default();
        db.expect_get_topics()
            .returning(|_page, _size| Ok(vec!["topic1".to_string(), "topic2".to_string()]));
        let app = init_service(App::new().service(get_topics).app_data(Data::new(db))).await;
        let req = TestRequest::with_uri("/?page=1").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<String> = read_body_json(resp).await;
        assert_eq!(body, vec!["topic1".to_string(), "topic2".to_string()]);
    }

    #[test]
    async fn test_add_topic() {
        let mut db = Database::default();
        db.expect_add_topic().returning(|_page| Ok(()));
        let app = init_service(App::new().service(add_topic).app_data(Data::new(db))).await;
        let topic = NewTopic {
            id: None,
            name: "topic1".to_string(),
        };
        let req = TestRequest::post().uri("/").set_json(&topic).to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: Health = read_body_json(resp).await;
        assert_eq!(
            body,
            Health::new(format!("Successfully created {}", topic.name))
        )
    }

    #[test]
    async fn test_add_topic_dup() {
        let mut db = Database::default();
        let response = "";
        let request = "";
        db.expect_add_topic()
            .returning(move |_page| Err(unexpected(response, request)));
        let app = init_service(App::new().service(add_topic).app_data(Data::new(db))).await;
        let topic = NewTopic {
            id: None,
            name: "topic1".to_string(),
        };
        let req = TestRequest::post().uri("/").set_json(&topic).to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = read_body(resp).await;
        assert_eq!(
            body,
            format!(
                "{}",
                actix_web::Error::from(Error::new(
                    unexpected(response, request),
                    StatusCode::INTERNAL_SERVER_ERROR
                ))
            )
        )
    }

    #[test]
    async fn test_delete_topic() {
        let mut db = Database::default();
        db.expect_delete_topic().returning(move |_page| Ok(()));
        let app = init_service(App::new().service(delete_topic).app_data(Data::new(db))).await;
        let topic = NewTopic {
            id: None,
            name: "topic1".to_string(),
        };
        let req = TestRequest::delete().uri("/").set_json(&topic).to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "testing success code");
        let body: Health = read_body_json(resp).await;
        assert_eq!(
            body,
            Health::new("Successfully deleted topic1".to_string()),
            "Testing success message"
        )
    }
}

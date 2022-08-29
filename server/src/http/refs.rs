use actix_web::web::{scope, Data, Json, Path, Query, ServiceConfig};
use actix_web::{get, post, services, Result};

#[mockall_double::double]
use crate::core::db::Database;
use crate::models::generic::{Generic, Pagination};
use crate::models::refs::{HRef, QRef, RefEnum};

pub fn refs_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/refs").service(services![get_references, add_qref, get_qrefs]));
}

#[get("/qref")]
async fn get_topics_for_qref(
    qref: Json<QRef>,
    q: Query<Pagination>,
    db: Data<Database>,
) -> Result<Json<Vec<String>>> {
    db.get_ref()
        .get_topics_from_qref(qref.0, q.page, q.size)
        .await
        .map_err(Into::into)
        .map(Json)
}

#[get("/{topic}")]
async fn get_references(topic: Path<String>, db: Data<Database>) -> Result<Json<Vec<RefEnum>>> {
    db.get_refs(&topic)
        .await
        .map(|v| Json(v.into_iter().filter(|r| !r.is_book()).collect()))
        .map_err(Into::into)
}

#[post("/{topic}/qref")]
async fn add_qref(topic: Path<String>, qref: Json<QRef>, db: Data<Database>) -> Result<Generic> {
    qref.validate()?;
    db.add_qref_to_topic(topic.as_str(), qref.0)
        .await
        .map(|_| Generic::new("Created Quran reference successfully".to_string()))
        .map_err(Into::into)
}

#[get("/{topic}/qref")]
async fn get_qrefs(
    topic: Path<String>,
    q: Query<Pagination>,
    db: Data<Database>,
) -> Result<Json<Vec<QRef>>> {
    db.get_qrefs(&topic, q.page, q.size)
        .await
        .map(Json)
        .map_err(Into::into)
}

#[post("/{topic}/href")]
async fn add_href(topic: Path<String>, href: Json<HRef>, db: Data<Database>) -> Result<Generic> {
    db.add_href_to_topic(topic.as_str(), href.0)
        .await
        .map(|_| Generic::new("Created Hadith reference successfully".to_string()))
        .map_err(Into::into)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::generic::Error;
    use actix_service::Service;
    use actix_web::{
        http::StatusCode,
        test,
        test::{init_service, read_body, read_body_json, TestRequest},
        App,
    };
    use aragog::error::Error as AError;
    use serde_json::to_string;

    #[test]
    async fn test_get_refs() {
        let mut db = Database::default();
        db.expect_get_refs()
            .returning(|_topic| Ok(Vec::<RefEnum>::new()));
        let app = init_service(App::new().service(get_references).app_data(Data::new(db))).await;
        let req = TestRequest::with_uri("/topic1").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<RefEnum> = read_body_json(resp).await;
        assert!(body.is_empty());
    }

    #[test]
    async fn test_add_qref() {
        let mut db = Database::default();
        db.expect_add_qref_to_topic()
            .returning(|_topic, _qref| Ok(()));
        let app = init_service(App::new().service(add_qref).app_data(Data::new(db))).await;
        let qref = QRef {
            chapter: 0,
            init_verse: 0,
            final_verse: 0,
        };
        let req = TestRequest::post()
            .uri("/topic1/qref")
            .set_json(&qref)
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let b: Generic = read_body_json(resp).await;
        assert_eq!(
            b,
            Generic::new("Created Quran reference successfully".to_string())
        )
    }

    #[test]
    async fn test_add_qref_invalid_topic() {
        let mut db = Database::default();
        let e = || {
            Err(Error::default(AError::NotFound {
                item: "".to_string(),
                id: "".to_string(),
                source: None,
            }))
        };
        db.expect_add_qref_to_topic()
            .returning(move |_topic, _qref| e());
        let app = init_service(App::new().service(add_qref).app_data(Data::new(db))).await;
        let qref = QRef {
            chapter: 0,
            init_verse: 0,
            final_verse: 0,
        };
        let req = TestRequest::post()
            .uri("/topic1/qref")
            .set_json(&qref)
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let b = read_body(resp).await;
        assert_eq!(b, to_string(&e().err()).unwrap());
    }

    #[test]
    async fn test_add_href() {
        let mut db = Database::default();
        db.expect_add_href_to_topic()
            .returning(|_topic, _href| Ok(()));
        let app = init_service(App::new().service(add_href).app_data(Data::new(db))).await;
        let href = HRef {
            collection: "".to_string(),
            number: "".to_string(),
        };
        let req = TestRequest::post()
            .uri("/topic1/href")
            .set_json(&href)
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let b: Generic = read_body_json(resp).await;
        assert_eq!(
            b,
            Generic::new("Created Hadith reference successfully".to_string())
        )
    }

    #[test]
    async fn test_add_href_invalid_topic() {
        let mut db = Database::default();
        let e = || {
            Err(Error::default(AError::NotFound {
                item: "topic1".to_string(),
                id: "Topic/topic1".to_string(),
                source: None,
            }))
        };
        db.expect_add_href_to_topic()
            .returning(move |_topic, _href| e());
        let app = init_service(App::new().service(add_href).app_data(Data::new(db))).await;
        let href = HRef {
            collection: "".to_string(),
            number: "".to_string(),
        };
        let req = TestRequest::post()
            .uri("/topic1/href")
            .set_json(&href)
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let b = read_body(resp).await;
        assert_eq!(b, to_string(&e().err()).unwrap());
    }
}

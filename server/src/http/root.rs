use actix_web::web::{scope, Data, ServiceConfig};
use actix_web::{get, services, Responder};

use mockall_double::double;

#[double]
use crate::core::db::Database;
use crate::models::generic::{Health, HealthStatus};

pub fn root_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("").service(services![health, root]));
}

#[get("/healthz")]
async fn health(db: Data<Database>) -> Result<Health, Health> {
    let mut health_status: Vec<HealthStatus> = vec![];

    let mut db_health = HealthStatus::new("Database".to_string());
    match db.health().await {
        Ok(_) => health_status.push(db_health),
        Err(e) => {
            db_health.set_status(e.to_string());
            db_health.set_unhealthy();
            health_status.push(db_health)
        }
    }

    let healthy_count = health_status.iter().filter(|h| h.is_healthy()).count();
    let health = Health::new(health_status);
    if healthy_count == 0 {
        Err(health)
    } else {
        Ok(health)
    }
}

#[get("/")]
async fn root() -> impl Responder {
    "Nothing to see here!"
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_service::Service;
    use actix_web::{
        http::StatusCode,
        test,
        test::{init_service, read_body, read_body_json, TestRequest},
        web::Bytes,
        App,
    };

    #[test]
    async fn test_root() {
        let app = init_service(App::new().service(root)).await;
        let req = TestRequest::with_uri("/").to_request();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body, Bytes::from_static(b"Nothing to see here!"))
    }

    // TODO: Update test to use new objects
    // #[test]
    // async fn test_health() {
    //     let mut db = Database::default();
    //     db.expect_health().returning(|| Ok(()));
    //     let app = init_service(App::new().service(health).app_data(Data::new(db))).await;
    //     let req = TestRequest::with_uri("/healthz").to_request();
    //     let resp = app.call(req).await.unwrap();
    //     assert_eq!(resp.status(), StatusCode::OK);
    //     let body: Health = read_body_json(resp).await;
    //     assert_eq!(body, Health::new("Everything is fine...".to_string()))
    // }

    // #[test]
    // async fn test_unhealthy() {
    //     let mut db = Database::default();
    //     db.expect_health()
    //         .returning(|| Err(NErr::AuthenticationError("".to_string())));
    //     let app = init_service(App::new().service(health).app_data(Data::new(db))).await;
    //     let req = TestRequest::with_uri("/healthz").to_request();
    //     let resp = app.call(req).await.unwrap();
    //     assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    //     let body = read_body(resp).await;
    //     let mut h: HashMap<&str, String> = HashMap::new();
    //     h.insert(
    //         "Database",
    //         format!("{:?}", NErr::AuthenticationError("".to_string())),
    //     );
    //     assert_eq!(
    //         body,
    //         format!(
    //             "{}",
    //             actix_web::Error::from(Error::new(
    //                 hash_to_health(h),
    //                 StatusCode::from_u16(400).unwrap()
    //             ))
    //         )
    //     )
    // }
}

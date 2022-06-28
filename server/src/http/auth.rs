use actix_web::web::{scope, Data, Query, ServiceConfig};
use actix_web::{get, post, services, HttpResponse, Responder};
use std::collections::HashMap;

use crate::core::auth::AuthHandler;
#[mockall_double::double]
use crate::core::db::Database;
use crate::models::generic::*;

pub fn auth_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").service(services![login, authorize]));
}

#[get("/login")]
async fn login(auth: Data<AuthHandler>, db: Data<Database>) -> Result<impl Responder, Error> {
    let url = auth.login(db.get_ref()).await?;
    url.host()
        .ok_or(Error::default("Failed to get login url"))
        .map(|h| {
            HttpResponse::Found()
                .append_header(("location", url.as_str()))
                .finish()
        })
}

#[get("/authorize")]
async fn authorize(q: Query<HashMap<String, String>>) -> String {
    for (k, v) in q.into_inner() {
        log::debug!("Key: {}, value: {}", k, v);
    }
    "nothing here also".to_string()
}

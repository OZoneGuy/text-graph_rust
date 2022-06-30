use actix_web::web::{scope, Data, Query, ServiceConfig};
use actix_web::{get, services, HttpResponse};
use serde::Deserialize;

use crate::core::auth::AuthHandler;
#[mockall_double::double]
use crate::core::db::Database;
use crate::models::generic::*;

pub fn auth_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").service(services![login, authorize]));
}

#[get("/login")]
async fn login(auth: Data<AuthHandler>, db: Data<Database>) -> Result<HttpResponse, Error> {
    let url = auth.login(db.get_ref()).await?;
    Ok(HttpResponse::Found()
        .append_header(("location", url.as_str()))
        .finish())
}

#[derive(Deserialize)]
struct AuthResponse {
    code: String,
    state: String,
}

#[get("/authorize")]
async fn authorize(
    q: Query<AuthResponse>,
    auth: Data<AuthHandler>,
    db: Data<Database>,
) -> Result<String, Error> {
    auth.get_ref()
        .validate_token(db.get_ref(), &q.code, &q.state)
        .await?;
    Ok(format!("Token: {}, state: {}", q.code, q.state))
}

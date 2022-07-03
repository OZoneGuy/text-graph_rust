use actix_web::cookie::Cookie;
use actix_web::web::{scope, Data, Query, ServiceConfig};
use actix_web::{get, services, HttpRequest, HttpResponse};
use actix_web_lab::middleware::from_fn;
use serde::{Deserialize, Serialize};

use crate::core::auth::AuthHandler;
#[mockall_double::double]
use crate::core::db::Database;
use crate::models::generic::*;

pub fn auth_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").service(services![login, authorize]));
}

#[derive(Deserialize)]
struct Referrer {
    referrer: Option<String>,
}

#[get("/login")]
async fn login(
    auth: Data<AuthHandler>,
    db: Data<Database>,
    referrer: Query<Referrer>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let r = referrer
        .0
        .referrer
        .unwrap_or_else(|| "/api/v1/".to_string());
    let (url, session) = auth.login(db.get_ref(), &r).await?;

    let host = req
        .headers()
        .get(actix_web::http::header::ORIGIN)
        .map(|hv| hv.to_str())
        .transpose()
        .map_err(Error::default)?;

    // Save session to cookie
    let session_cookie = Cookie::build("ir_session", session)
        .domain(host.unwrap_or("localhost"))
        .path("/api/v1/")
        .secure(true)
        .finish();
    // Redirect to login
    Ok(HttpResponse::Found()
        .append_header(("location", url.as_str()))
        .cookie(session_cookie)
        .finish())
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    code: String,
    state: String,
}

#[get("/authorize")]
async fn authorize(
    q: Query<AuthResponse>,
    auth: Data<AuthHandler>,
    db: Data<Database>,
) -> Result<HttpResponse, Error> {
    let state_split: Vec<&str> = q.state.split('&').collect(); // First is the ID and the second element is the referrer
    let state = state_split
        .get(0)
        .ok_or_else(|| Error::default("Failed to get the state"))?
        .trim_start_matches("State=");
    let referrer = state_split
        .get(1)
        .unwrap_or(&"/api/v1/")
        .trim_start_matches("Referrer=");
    auth.get_ref()
        .get_token(db.get_ref(), &q.code, state)
        .await?;
    Ok(HttpResponse::Found()
        .append_header(("location", referrer))
        .finish())
}

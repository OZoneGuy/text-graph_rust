use actix_identity::Identity;
use actix_web::web::{scope, Data, Form, Json, Query, ServiceConfig};
use actix_web::{get, post, services, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::core::auth::AuthHandler;
#[mockall_double::double]
use crate::core::db::Database;
use crate::models::auth::User;
use crate::models::generic::*;

pub fn auth_service(cfg: &mut ServiceConfig) {
    cfg.service(scope("/auth").service(services![login, authorize, user]));
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
) -> Result<HttpResponse, Error> {
    let r = referrer
        .0
        .referrer
        .unwrap_or_else(|| "/api/v1/".to_string());
    let url = auth.login(db.get_ref(), &r).await?;

    // Redirect to login
    Ok(HttpResponse::Found()
        .append_header(("location", url.as_str()))
        .finish())
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    id_token: String,
    state: String,
}

#[post("/authorize")]
async fn authorize(
    q: Form<AuthResponse>,
    auth: Data<AuthHandler>,
    db: Data<Database>,
    id: Identity,
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
        .validate_token(db.get_ref(), &q.id_token, state)
        .await?;
    id.remember(state.to_string());
    Ok(HttpResponse::Found()
        .append_header(("location", referrer))
        .finish())
}

#[get("/user")]
async fn user(
    auth: Data<AuthHandler>,
    db: Data<Database>,
    id: Identity,
) -> Result<Json<User>, Error> {
    let session: String = id
        .identity()
        .ok_or_else(|| Error::new("Not logged in!", actix_web::http::StatusCode::UNAUTHORIZED))?;
    auth.get_ref()
        .get_user(db.get_ref(), session)
        .await
        .map(Json)
}

#[macro_use] extern crate rocket;
use rocket::serde::json::{ Json };

#[launch]
fn rocker() -> _ {
    rocket::build().mount("/", routes![health, root])
}

mod models;
use models::Health;

#[get("/healthz")]
fn health() -> Json<Health> {
    Json(Health::new("Everything is fine...".to_string()))
}

#[get("/")]
fn root() -> String {
    "Nothing to see here!".to_string()
}

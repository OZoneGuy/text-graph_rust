use rocket::request::{ FromRequest, Outcome, Request };
use rocket::http::{ Cookie, Status };

pub struct AuthHandler {
}

#[derive(Debug)]
pub enum AuthError {
    // PrivilegeError,
    Unauthorised,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHandler {
    type Error = AuthError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if request.uri().path() == "/login" {
            request.cookies().add(Cookie::new("auth", "auth"));

            return Outcome::Success(AuthHandler{});
        }

        if let Some(_) = request.cookies().get("auth") {
            Outcome::Success(AuthHandler{})
        } else {
            Outcome::Failure((Status::new(403), AuthError::Unauthorised ))
        }
    }
}

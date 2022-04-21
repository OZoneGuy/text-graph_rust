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
            debug!("Entering Auth guard in /login endpoint");
            request.cookies().add(Cookie::new("auth", "auth"));

            return Outcome::Success(AuthHandler{});
        }

        debug!("Running the auth gaurd on a different endpoint");
        if let Some(_) = request.cookies().get("auth") {
            debug!("Successful authentication");
            Outcome::Success(AuthHandler{})
        } else {
            debug!("Failed to authenticate");
            Outcome::Failure((Status::new(403), AuthError::Unauthorised ))
        }
    }
}

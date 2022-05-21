pub struct AuthHandler {}

#[derive(Debug)]
pub enum AuthError {
    // PrivilegeError,
    Unauthorised,
}

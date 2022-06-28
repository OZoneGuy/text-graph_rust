#[mockall_double::double]
use super::db::Database;
use crate::models::generic::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use aragog::Record;
use core::future::Future;
use oauth2::reqwest::http_client;
use oauth2::{
    basic::BasicClient, AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct AuthHandler {
    client: BasicClient,
}

#[derive(Serialize, Deserialize, Clone, Record, Debug, PartialEq)]
pub struct SessionRecord {
    #[serde(rename = "_key")]
    pub key: Option<String>,
    pub session: Option<String>,
    pub verifier: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AuthError {
    // PrivilegeError,
    Unauthorised,
}

impl AuthHandler {
    pub fn new(host: String, client_secret: String, client_id: String) -> Self {
        let base_url: String = "https://login.microsoftonline.com/common/oauth2/v2.0".to_string();
        let auth_url =
            AuthUrl::new(format!("{}/authorize", base_url)).expect("Failed to create AuthUrl");
        let token_url =
            TokenUrl::new(format!("{}/token", base_url)).expect("Failed to create TokenUrl");
        let client = BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/api/v1/auth/authorize", host))
                .expect("Failed to create redirect url"),
        );
        AuthHandler { client }
    }

    pub async fn login(&self, db: &Database) -> Result<oauth2::url::Url, Error> {
        let rand_state: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (u, _csrf_token) = self
            .client
            .authorize_url(|| CsrfToken::new(rand_state.clone()))
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            // .add_extra_param("state", &rand_state)
            .set_pkce_challenge(pkce_challenge)
            .url();
        db.add_session(SessionRecord {
            key: Some(rand_state),
            session: None,
            verifier: pkce_verifier.secret().clone(),
        })
        .await
        .map(|_| u)
    }

    pub async fn validate_token(
        &self,
        db: Database,
        token: String,
        state: String,
    ) -> Result<AccessToken, Error> {
        // 1. Retreive the pkce verifier, using the state
        let s = db.get_session(state).await?;

        // 2. Get the token result
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(token))
            .set_pkce_verifier(PkceCodeVerifier::new(s.verifier.clone()))
            .request(http_client)
            .map_err(Error::default)?;

        // 3. Return the Access token. To be saved in the session
        // 3a. Save the token information in the database?
        db.update_session(
            s.key.expect("Didn't get the required key"),
            token_result.access_token().secret().clone(),
        )
        .await
        .map_err(Error::default)
        .map(|s| {
            oauth2::AccessToken::new(
                s.session
                    .expect("Failed to set the session token and retrieve it"),
            )
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthHandler
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleWare<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let auth = (*self).clone();
        ready(Ok(AuthMiddleWare { auth, service }))
    }
}

pub struct AuthMiddleWare<S> {
    auth: AuthHandler,
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleWare<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    // Should check if the user is logged in here or not. Otherwise, redirect to mocrosoft for login
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move { Ok(fut.await?) })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{from_value, json, to_string};
    use std::borrow::Cow;

    #[test]
    fn serialize_session_struct() {
        let s = SessionRecord {
            key: Some("Random".to_string()),
            session: None,
            verifier: "Verifier".to_string(),
        };
        let val = json!({
            "_key": "Random",
            "session": null,
            "verifier": "Verifier"
        });
        assert!(serde_json::to_string(&s).is_ok());
        assert_eq!(to_string(&s).unwrap(), to_string(&val).unwrap());
        assert_eq!(from_value::<SessionRecord>(val).unwrap(), s)
    }

    #[test]
    fn test_new() {
        let auth = AuthHandler::new(
            "http://localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
    }

    #[test]
    #[should_panic]
    fn test_new_bad_host() {
        let auth = AuthHandler::new(
            "localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
    }

    #[actix_web::test]
    async fn get_login_url() {
        let mut db = Database::default();
        db.expect_add_session().returning(|_| Ok(()));
        let auth = AuthHandler::new(
            "http://localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
        let url_result = auth.login(&db).await;
        assert!(url_result.is_ok(), "Created auth url successfully");
        let url = url_result.unwrap();
        assert_eq!(
            url.path(),
            "/common/oauth2/v2.0/authorize",
            "Auth path is correct"
        );
        assert!(url.query().is_some(), "Auth url has queries");
        assert!(
            url.query_pairs()
                .find(|q| q == &(Cow::Borrowed("client_id"), Cow::Borrowed("id")))
                .is_some(),
            "Auth url specifies the correct client_id"
        );
        assert!(
            url.query_pairs()
                .find(|q| q
                    == &(
                        Cow::Borrowed("redirect_uri"),
                        Cow::Borrowed("http://localhost/api/v1/auth/authorize")
                    ))
                .is_some(),
            "Auth url specifies the correct redirect uri"
        )
    }
}

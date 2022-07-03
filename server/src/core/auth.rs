#[mockall_double::double]
use super::db::Database;
use crate::models::generic::Error;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use aragog::Record;
use core::future::Future;
use oauth2::reqwest::async_http_client;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
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
    pub verifier: String,
    pub token: Option<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: String,
    pub token: String,
    pub creation_date: std::time::SystemTime,
    pub expiration: Option<core::time::Duration>,
    pub refresh_token: Option<String>,
}

impl AuthHandler {
    pub fn new(host: String, client_secret: String, client_id: String) -> Self {
        let base_url: String = "https://login.microsoftonline.com/common/oauth2/v2.0".to_string();
        let auth_url =
            AuthUrl::new(format!("{}/authorize", base_url)).expect("Failed to create AuthUrl");
        let token_url =
            TokenUrl::new(format!("{}/token", base_url)).expect("Failed to create TokenUrl");
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/api/v1/auth/authorize", host))
                .expect("Failed to create redirect url"),
        );
        AuthHandler { client }
    }

    pub async fn login(
        &self,
        db: &Database,
        referrer: &str,
    ) -> Result<(oauth2::url::Url, String), Error> {
        // Create random state
        // To be saved in the browser
        let rand_state: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        // get a verifier for the code token
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Save session in the database
        db.add_session(
            rand_state.clone(),
            SessionRecord {
                token: None,
                verifier: pkce_verifier.secret().clone(),
            },
        )
        .await?;

        // Create URL to get auth code
        let (u, _csrf_token) = self
            .client
            .authorize_url(|| {
                CsrfToken::new(format!(
                    "State={}&Referrer={}",
                    rand_state.clone(),
                    referrer
                ))
            })
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        // Return url and state
        Ok((u, rand_state))
    }

    pub async fn get_token(&self, db: &Database, code: &str, state: &str) -> Result<(), Error> {
        // 1. Retreive the pkce verifier, using the state
        let s = db.get_session(state.to_string()).await?;

        // 2. Get the token result
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(s.verifier.clone()))
            .request_async(async_http_client)
            .await
            .map_err(Error::default)?;

        // 3. Return the Access token. To be saved in the session
        // 3a. Save the token information in the database?
        let token: Token = Token {
            token_type: token_result.token_type().as_ref().to_string(),
            token: token_result.access_token().secret().to_string(),
            expiration: token_result.expires_in(),
            creation_date: std::time::SystemTime::now(),
            refresh_token: token_result
                .refresh_token()
                .map(|r| oauth2::RefreshToken::secret(r).clone()),
        };
        db.update_session(state.to_string(), token)
            .await
            .map_err(Error::default)
            .map(|_| ())
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
        Box::pin(async move { fut.await })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_new() {
        let _auth = AuthHandler::new(
            "http://localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
    }

    #[test]
    #[should_panic]
    fn test_new_bad_host() {
        let _auth = AuthHandler::new(
            "localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
    }

    #[actix_web::test]
    async fn get_login_url() {
        let mut db = Database::default();
        db.expect_add_session().returning(|_, _| Ok(()));
        let auth = AuthHandler::new(
            "http://localhost".to_string(),
            "secret".to_string(),
            "id".to_string(),
        );
        let url_result = auth.login(&db, "base").await;
        assert!(url_result.is_ok(), "Created auth url successfully");
        let (url, _) = url_result.unwrap();
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

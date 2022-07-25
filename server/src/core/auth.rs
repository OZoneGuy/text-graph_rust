#[mockall_double::double]
use super::db::Database;
use crate::models::{auth::*, generic::Error};
use actix_identity::RequestIdentity;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::Error as AError,
    HttpResponse,
};
use actix_web_lab::middleware::Next;
use chrono::prelude::*;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use rand::{distributions::Alphanumeric, Rng};
use ureq::get;
use url::Url;

// #[derive(Debug)]
pub struct AuthHandler {
    #[allow(dead_code)]
    client_secret: String,
    client_id: String,
    host: String,
}

impl AuthHandler {
    pub fn new(host: String, client_secret: String, client_id: String) -> Self {
        AuthHandler {
            client_secret,
            client_id,
            host,
        }
    }

    pub async fn login(&self, db: &Database, referrer: &str) -> Result<url::Url, Error> {
        // Create random state
        // To be saved in the browser
        let session: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // create nonce
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        // Save session in the database
        db.add_session(
            session.clone(),
            SessionRecord {
                token: None,
                nonce: nonce.clone(),
            },
        )
        .await?;
        // Create URL to get auth code
        let u = Url::parse_with_params(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            &[
                ("client_id", self.client_id.as_str()),
                ("response_type", "id_token"),
                (
                    "redirect_uri",
                    &format!("{}/api/v1/auth/authorize", self.host),
                ),
                ("response_mode", "form_post"),
                ("scope", "openid profile email"),
                ("state", &format!("State={}&Referrer={}", session, referrer)),
                ("nonce", &nonce),
            ],
        )
        .map_err(Error::default)?;

        // Return url and state
        Ok(u)
    }

    pub async fn validate_token(&self, db: &Database, jwt: &str, state: &str) -> Result<(), Error> {
        // 1. Retreive the pkce verifier, using the state
        let s = db.get_session(state.to_string()).await?;

        // 2 decode jwt
        // 2.a get the kId from the header
        let jwt_header = decode_header(jwt).map_err(Error::default)?;
        let kid = jwt_header.kid.expect("jwt header should have a kid");

        // 2.b retrieve the key from the ms endpoint using the kId
        let key = get("https://login.microsoftonline.com/common/discovery/v2.0/keys")
            .set("kid", &kid)
            .call()
            .map_err(Error::default)?
            .into_json::<jsonwebtoken::jwk::JwkSet>()
            .map_err(Error::default)?
            .keys
            .iter()
            .find(|k| k.common.key_id == Some(kid.clone()))
            .ok_or_else(|| Error::default("JWKS is empty"))?
            .clone();
        let decode_key: DecodingKey = match key.algorithm {
            jsonwebtoken::jwk::AlgorithmParameters::RSA(p) => {
                Ok(DecodingKey::from_rsa_components(&p.n, &p.e).map_err(Error::default)?)
            }
            _ => Err(Error::default("Unsupported alogirthm")),
        }?;

        // 2.c actually decode the jwt...
        let validator = Validation::new(jwt_header.alg);
        let jwt_decoded =
            decode::<SessionRecord>(jwt, &decode_key, &validator).map_err(Error::default)?;

        // 2.d Validate the jwt using the nonce
        if jwt_decoded.claims.nonce != s.nonce {
            return Err(Error::default("Failed to validate jwt nonce"));
        };

        let mut session_record = jwt_decoded.claims;
        session_record
            .token
            .as_mut()
            .expect("Failed to parse token")
            .token = Some(jwt.to_string());

        // 3. Return the Access token. To be saved in the session
        // 3a. Save the token information in the database?
        db.update_session(
            state.to_string(),
            session_record.token.expect("Failed to parse token"),
        )
        .await
        .map_err(Error::default)
        .map(|_| ())
    }

    pub async fn is_logged_in(&self, db: &Database, session: String) -> Result<bool, Error> {
        db.get_session(session).await?.token.map_or(Ok(false), |t| {
            log::debug!("{:?}", t.exp);
            Ok(t.exp.cmp(&Utc::now()).is_gt())
        })
    }

    pub async fn auth_middleware(
        req: ServiceRequest,
        next: Next<impl MessageBody + 'static>,
    ) -> Result<ServiceResponse<impl MessageBody>, AError> {
        // get session cookie
        let session_cookie = req.get_identity();
        if session_cookie.is_none() {
            let resp = HttpResponse::Unauthorized().finish().map_into_boxed_body();
            let (request, _) = req.into_parts();
            return Ok(ServiceResponse::new(request, resp));
        };

        let session_str = session_cookie.unwrap();

        // get db
        let db = req
            .app_data::<actix_web::web::Data<Database>>()
            .ok_or_else(|| Error::default("Unable to get Database"))?;

        // Get AuthHandler
        let auth_handler = req
            .app_data::<actix_web::web::Data<AuthHandler>>()
            .ok_or_else(|| Error::default("Unable to get AuthHandler"))?;

        // authenticate user
        if auth_handler.is_logged_in(db, session_str).await? {
            next.call(req)
                .await
                .map(ServiceResponse::map_into_boxed_body)
        } else {
            let resp = HttpResponse::Unauthorized().finish().map_into_boxed_body();
            let (request, _) = req.into_parts();
            Ok(ServiceResponse::new(request, resp))
        }
    }

    #[allow(dead_code, unused_variables)]
    pub async fn authorize_middleware(
        req: ServiceRequest,
        next: Next<impl MessageBody + 'static>,
    ) -> Result<ServiceResponse<impl MessageBody>, AError> {
        // get session cookie
        let session_cookie = req.cookie("ir_session");
        if session_cookie.is_none() {
            let resp = HttpResponse::Found()
                .append_header((
                    "location",
                    format!("/api/v1/auth/login?referrer={}", req.path()),
                ))
                .finish()
                .map_into_boxed_body();
            let (request, _) = req.into_parts();
            return Ok(ServiceResponse::new(request, resp));
        };

        let session_str = session_cookie.unwrap().value().to_string();

        // get db
        let db = req
            .app_data::<actix_web::web::Data<Database>>()
            .ok_or_else(|| Error::default("Unable to get Database"))?;

        // Get AuthHandler
        let auth_handler = req
            .app_data::<actix_web::web::Data<AuthHandler>>()
            .ok_or_else(|| Error::default("Unable to get AuthHandler"))?;
        let session = db.get_session(session_str).await?;
        next.call(req)
            .await
            .map(ServiceResponse::map_into_boxed_body)
    }

    pub async fn get_user(&self, db: &Database, session: String) -> Result<User, Error> {
        if !self.is_logged_in(db, session.clone()).await? {
            return Err(Error::new(
                "User is not logged in",
                actix_web::http::StatusCode::UNAUTHORIZED,
            ));
        };
        let t = db
            .get_session(session)
            .await?
            .token
            .expect("Token is not present");
        Ok(User {
            name: t.name,
            email: t.preferred_username,
        })
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
    #[ignore]
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

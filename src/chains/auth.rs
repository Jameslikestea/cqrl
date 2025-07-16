use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{task, time};

use crate::chains::{
    keys::{AUTH_CONTEXT_ID_KEY, AUTH_CONTEXT_TYPE_KEY},
    ChainLink,
};

enum AuthType {
    AppUser(String),
    #[allow(dead_code)]
    ApiKey(String),
    Unauthenticated,
}

#[allow(dead_code)]
pub(crate) struct AuthChain {
    jwks: Arc<Mutex<jsonwebtoken::jwk::JwkSet>>,
}

impl AuthChain {
    #[allow(dead_code)]
    pub(crate) fn new(jwks_url: Option<String>) -> Self {
        let jwks_arc = Arc::new(Mutex::new(JwkSet { keys: vec![] }));

        let jwks_arc_clone = jwks_arc.clone();

        match jwks_url {
            Some(url) => {
                let url = url.clone();
                task::spawn(async move {
                    let mut interval = time::interval(Duration::from_secs(300));

                    loop {
                        let res: reqwest::Response = reqwest::get(url.clone()).await.unwrap();
                        let bytes = res.bytes().await.unwrap();

                        let b = bytes.iter().map(|b| *b).collect::<Vec<u8>>();
                        let jwks_str = String::from_utf8(b).unwrap();

                        let jwks: jsonwebtoken::jwk::JwkSet =
                            serde_json::from_str(jwks_str.as_str()).unwrap();

                        tracing::info!(
                            count = jwks.keys.len(),
                            ids = jwks
                                .keys
                                .iter()
                                .map(|k| k.common.key_id.clone().unwrap_or("unknown".to_string()))
                                .collect::<Vec<String>>()
                                .join(", "),
                            "got jwks from server"
                        );

                        *jwks_arc.lock().unwrap() = jwks;

                        interval.tick().await;
                    }
                });
            }
            None => {}
        };

        Self {
            jwks: jwks_arc_clone,
        }
    }

    fn get_auth_type(&self, auth_header: &str) -> AuthType {
        let auth = auth_header.split(" ").collect::<Vec<&str>>();

        if auth.len() != 2 {
            tracing::info!("invalid auth header: {:?}", auth_header);
            return AuthType::Unauthenticated;
        }

        if auth[0].to_string() == "Bearer" {
            tracing::info!("app user auth header: {:?}", auth[1]);
            return AuthType::AppUser(auth[1].to_string());
        }

        if auth[0].to_string() == "Api-Key" {
            tracing::info!("api key auth header: {:?}", auth[1]);
            return AuthType::ApiKey(auth[1].to_string());
        }

        AuthType::Unauthenticated
    }

    async fn validate_auth(
        &self,
        auth: AuthType,
        ctx: ContextManager<String, String>,
    ) -> ContextManager<String, String> {
        match auth {
            AuthType::AppUser(user) => {
                let mut ctx = ctx.clone();
                match self.decode_token(&user).await {
                    Ok(claims) => {
                        tracing::info!("app user claims: {:?}", claims);
                        ctx.insert(AUTH_CONTEXT_ID_KEY.to_string(), claims.sub.to_string());
                        ctx.insert(AUTH_CONTEXT_TYPE_KEY.to_string(), "app_user".to_string());
                    }
                    Err(e) => {
                        tracing::warn!("cannot decode token: {:?}", e);
                        ctx.insert(
                            AUTH_CONTEXT_TYPE_KEY.to_string(),
                            "unauthenticated".to_string(),
                        );
                    }
                };

                ctx
            }
            _ => {
                let mut ctx = ctx.clone();
                ctx.insert(
                    AUTH_CONTEXT_TYPE_KEY.to_string(),
                    "unauthenticated".to_string(),
                );
                ctx
            }
        }
    }

    async fn decode_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let hdr = match decode_header(token) {
            Ok(hdr) => hdr,
            Err(_) => return Err("invalid token: no header".into()),
        };

        let kid = hdr.kid.unwrap();
        let jwks = self.jwks.lock().unwrap();

        let jwk = match jwks.find(&kid) {
            None => return Err("invalid token: key not found".into()),
            Some(jwk) => jwk,
        };

        let mut validation = Validation::new(hdr.alg);
        validation.set_audience(&vec!["cqrl".to_string()]);

        let claims = decode::<Claims>(token, &DecodingKey::from_jwk(jwk).unwrap(), &validation);

        match claims {
            Ok(claims) => Ok(claims.claims),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
}

#[async_trait(?Send)]
impl ChainLink for AuthChain {
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: Arc<HttpRequest>,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        tracing::info!("auth chain");

        let auth_hdr = match _request.headers().get("Authorization") {
            Some(auth_header) => auth_header.to_str().unwrap(),
            None => "",
        };

        tracing::info!("auth header: {:?}", auth_hdr);

        let auth_info = self.get_auth_type(auth_hdr);

        let ctx = self.validate_auth(auth_info, context.clone()).await;

        tracing::info!("auth context: {:?}", ctx);

        Ok(ctx)
    }
}

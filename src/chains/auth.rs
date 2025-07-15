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
use tracing::warn;

use crate::chains::{
    keys::{AUTH_CONTEXT_ID_KEY, AUTH_CONTEXT_TYPE_KEY},
    ChainLink,
};

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
        let auth_type = match _request.headers().get("Authorization") {
            Some(auth_header) => {
                let auth = auth_header
                    .to_str()
                    .unwrap()
                    .split(" ")
                    .collect::<Vec<&str>>();
                if auth_header.to_str().unwrap().starts_with("Bearer") {
                    ("app_user", auth[1].to_string())
                } else if auth_header.to_str().unwrap().starts_with("Api-Key") {
                    ("api_key", auth[1].to_string())
                } else {
                    ("unauthenticated", "".to_string())
                }
            }
            None => ("unauthenticated", "".to_string()),
        };
        let mut ctx = context.clone();

        match auth_type {
            ("app_user", user) => {
                let hdr = match decode_header(&user) {
                    Err(_) => return Err("invalid token: no header".into()),
                    Ok(hdr) => hdr,
                };
                let kid = hdr.kid.unwrap();
                tracing::info!("decoding token with kid: {}, alg: {:?}", &kid, &hdr.alg);
                let jwks = self.jwks.lock().unwrap();

                let jwk = match jwks.find(&kid) {
                    None => {
                        warn!("jwk not found for kid: {}", &kid);
                        return Err("invalid token: key not found".into());
                    }
                    Some(jwk) => jwk,
                };

                let mut validation = Validation::new(hdr.alg);
                validation.set_audience(&vec!["cqrl".to_string()]);

                let claims = decode::<Claims>(
                    user.as_str(),
                    &DecodingKey::from_jwk(jwk).unwrap(),
                    &validation,
                );

                match claims {
                    Ok(claims) => {
                        ctx.insert(AUTH_CONTEXT_ID_KEY.to_string(), claims.claims.sub);
                    }
                    Err(e) => {
                        warn!("invalid token: cannot decode claims: {}", e);
                        return Err("invalid token: cannot decode claims".into());
                    }
                }

                ctx.insert(AUTH_CONTEXT_TYPE_KEY.to_string(), "app_user".to_string());
            }
            ("api_key", key) => {
                ctx.insert(AUTH_CONTEXT_TYPE_KEY.to_string(), "api_key".to_string());
            }
            (_, _) => {
                ctx.insert(
                    AUTH_CONTEXT_TYPE_KEY.to_string(),
                    "unauthenticated".to_string(),
                );
            }
        }

        Ok(ctx)
    }
}

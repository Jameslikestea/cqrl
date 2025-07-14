use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};

use actix_web::HttpRequest;
use async_trait::async_trait;
use chrono::Utc;
use contexts::ContextManager;
use mongodb::{
    bson::{self, doc},
    options::IndexOptions,
    Client, IndexModel,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, info, warn};

use crate::chains::{keys::REQUEST_HEADER_IP_KEY, ChainLink};

pub(crate) struct RateLimitChain {
    store: Arc<Client>,
}

impl RateLimitChain {
    pub(crate) fn new(store: Arc<Client>) -> Self {
        let ratelimit_store = store.clone();
        tokio::spawn(async move {
            let collection = ratelimit_store
                .database("cqrl")
                .collection::<RateLimit>("ratelimits");

            match collection
                .create_index(
                    IndexModel::builder()
                        .keys(doc! { "expiresAfter": 1 })
                        .options(Some(
                            IndexOptions::builder()
                                .expire_after(Duration::from_secs(60))
                                .build(),
                        ))
                        .build(),
                )
                .await
            {
                Ok(_) => info!("successfully created rate limit collection"),
                Err(e) => warn!("error creating rate limit collection: {}", e),
            };
        });

        Self { store }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RateLimit {
    ip: String,
    current: u32,
}

#[derive(Debug)]
pub enum RateLimitError {
    RateLimitExceeded,
}

impl Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RateLimitError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl Error for RateLimitError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            RateLimitError::RateLimitExceeded => None,
        }
    }

    fn description(&self) -> &str {
        match self {
            RateLimitError::RateLimitExceeded => "Rate limit exceeded",
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RateLimitError::RateLimitExceeded => None,
        }
    }
}

#[async_trait(?Send)]
impl ChainLink for RateLimitChain {
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: Arc<HttpRequest>,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn Error>> {
        if let Some(ip) = context.get(REQUEST_HEADER_IP_KEY) {
            let ip = ip.to_string();
            let collection = self
                .store
                .database("cqrl")
                .collection::<RateLimit>("ratelimits");
            let result: Option<RateLimit> = collection.find_one(doc! { "ip": ip.clone() }).await?;
            if let Some(doc) = result {
                let current = doc.current;
                if current >= 100 {
                    warn!("rate limit exceeded for ip: {}", ip.clone());
                    return Err(Box::new(RateLimitError::RateLimitExceeded));
                }
            }

            info!("rate limit for ip: {}", ip.clone());

            let expires_after = Utc::now();
            let expires_after_value = bson::DateTime::from_millis(expires_after.timestamp_millis());

            match collection
                .update_one(
                    doc! { "ip": ip.clone() },
                    doc! { "$setOnInsert": { "ip": ip.clone(), "expiresAfter": expires_after_value }, "$inc": { "current": 1 } },
                )
                .upsert(true)
                .await
            {
                Ok(_) => debug!("rate limit updated for ip: {}", ip.clone()),
                Err(e) => warn!(
                    "failed to update rate limit for ip: {} -> {}",
                    ip.clone(),
                    e
                ),
            };
        }
        Ok(context.clone())
    }
}

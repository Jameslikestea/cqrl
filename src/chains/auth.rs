use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use serde_json::Value;

use crate::chains::{keys::AUTH_CONTEXT_TYPE_KEY, ChainLink};

#[allow(dead_code)]
pub(crate) struct AuthChain {}

impl AuthChain {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[async_trait(?Send)]
impl ChainLink for AuthChain {
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut ctx = context.clone();
        ctx.insert(
            AUTH_CONTEXT_TYPE_KEY.to_string(),
            "unauthenticated".to_string(),
        );
        Ok(ctx)
    }
}

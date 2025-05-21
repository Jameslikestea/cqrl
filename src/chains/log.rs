
use actix_web::{
    FromRequest, HttpRequest,
    dev::ConnectionInfo,
};
use async_trait::async_trait;
use contexts::ContextManager;
use serde_json::Value;
use tracing::instrument;

use super::ChainLink;

pub(crate) struct LogChain;

#[async_trait(?Send)]
impl ChainLink for LogChain {
    #[instrument(skip(self, context, request, _body) name = "log_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let connection_info = ConnectionInfo::extract(request).await.unwrap();

        let method = request.method().as_str();
        let path = request.path();

        let ip = connection_info.realip_remote_addr().unwrap_or("unknown");

        tracing::info!(ip = ip, "{method} {path}");

        Ok(context.clone())
    }
}

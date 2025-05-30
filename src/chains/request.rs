use std::collections::HashMap;

use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use serde_json::Value;

use super::{keys::REQUEST_HEADER_IF_NONE_MATCH_KEY, ChainLink};

/**
 * This chain is used to extract the headers that the application cares about from the request
 * and load them into the context for later processing.
 */
pub (crate) struct HeaderChain;

#[async_trait(?Send)]
impl ChainLink for HeaderChain {
    async fn process(&self, context: &ContextManager<String, String>, request: &HttpRequest, _: &Value) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut hm = HashMap::new();

        let headers = request.headers();
        for etags in headers.get_all("If-None-Match") {
            hm.insert(
                REQUEST_HEADER_IF_NONE_MATCH_KEY.to_string(),
                etags.to_str().unwrap().to_string(),
            );
        }

        context.push(hm);

        Ok(context)
    }
}

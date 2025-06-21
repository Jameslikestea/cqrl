use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{
    http::Method, web::{Form, Json}, Either, Handler, HttpRequest, HttpResponse
};
use async_trait::async_trait;
use contexts::ContextManager;
use keys::{REQUEST_HEADER_IF_NONE_MATCH_KEY, RESPONSE_DATA_KEY, RESPONSE_HEADER_COMMAND_KEY, RESPONSE_HEADER_ETAG_KEY};
use opentelemetry::{global, metrics::{Counter, Histogram}, KeyValue};
use serde_json::Value;
use tracing::instrument;

pub(crate) mod keys;
pub(crate) mod log;
pub(crate) mod methods;
pub(crate) mod persistence;
pub(crate) mod url;
pub(crate) mod request;

#[derive(Clone)]
pub(crate) struct ProcessingChain {
    links: Vec<Arc<dyn ChainLink>>,
    response_size_histogram: Arc<Histogram<u64>>,
    requests: Arc<Counter<u64>>,
}

impl ProcessingChain {
    pub(crate) fn new(links: Vec<Arc<dyn ChainLink>>) -> Self {
        let meter = global::meter("cqrl-server");
        let response_size_histogram = meter.u64_histogram("cqrl_processing_chain_response_size").build();
        let requests = meter.u64_counter("cqrl_processing_chain_requests").build();
        Self { links, response_size_histogram: Arc::new(response_size_histogram), requests: Arc::new(requests) }
    }
}

impl Handler<(HttpRequest, Option<Either<Json<Value>, Form<Value>>>)> for ProcessingChain {
    type Output = HttpResponse;
    type Future = Pin<Box<dyn Future<Output = HttpResponse>>>;

    #[instrument(skip(self, request, body) name = "processing_chain")]
    fn call(
        &self,
        (request, body): (HttpRequest, Option<Either<Json<Value>, Form<Value>>>),
    ) -> Self::Future {
        let links: Vec<Arc<dyn ChainLink + 'static>> = self.links.clone();
        self.requests.add(1, &[KeyValue::new("method", request.method().to_string())]);

        let body = match body {
            Some(Either::Left(body)) => body.into_inner(),
            Some(Either::Right(body)) => body.into_inner(),
            None => Value::Null,
        };
        let response_size_histogram = self.response_size_histogram.clone();
        Box::pin(async move {
            let mut context = ContextManager::new();

            for link in links.iter() {
                context = match link.process(&context, &request, &body).await {
                    Ok(context) => context,
                    Err(e) => {
                        return HttpResponse::InternalServerError().body(e.to_string());
                    }
                };
            }

            if context.get(RESPONSE_DATA_KEY).is_some() {
                let Some(response_data) = context.get(RESPONSE_DATA_KEY) else {
                    let content = "No response data found".to_string();
                    response_size_histogram.record(content.len() as u64, &[KeyValue::new("status", "500")]);
                    return HttpResponse::InternalServerError()
                        .body(content);
                };

                let mut builder = HttpResponse::Ok();
                
                if let Some(etag) = context.get(RESPONSE_HEADER_ETAG_KEY) {
                    if let Some(request_etag) = context.get(REQUEST_HEADER_IF_NONE_MATCH_KEY) {
                        if request_etag.to_string() == etag.to_string() {
                            response_size_histogram.record(0, &[KeyValue::new("status", "304")]);
                            return HttpResponse::NotModified().body("");
                        }
                    }
                    
                    builder = builder.append_header(("ETag", etag.to_string())).append_header(("Cache-Control", "private, max-age=30")).take();
                }

                if request.method() == Method::HEAD {
                    response_size_histogram.record(0, &[KeyValue::new("status", "200")]);
                    return builder.append_header(("Access-Control-Allow-Origin", "*")).append_header(("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS, ")).body("");
                }

                response_size_histogram.record(response_data.len() as u64, &[KeyValue::new("status", "200")]);
                let response_data: serde_json::Value = serde_json::from_str(response_data).unwrap();
                return builder.json(response_data);
            }

            let mut builder = HttpResponse::Accepted();

            if let Some(command_header) = context.get(RESPONSE_HEADER_COMMAND_KEY) {
                builder = builder.append_header(("X-Command-Id", command_header.to_string())).take();
            }

            response_size_histogram.record(0, &[KeyValue::new("status", "202")]);
            builder.body("")
        })
    }
}
#[async_trait(?Send)]
pub(crate) trait ChainLink: Send + Sync {
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        Ok(context.clone())
    }
}

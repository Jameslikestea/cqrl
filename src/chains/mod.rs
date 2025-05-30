use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{
    Either, Handler, HttpRequest, HttpResponse,
    web::{Form, Json},
};
use async_trait::async_trait;
use contexts::ContextManager;
use keys::{RESPONSE_DATA_KEY, RESPONSE_HEADER_COMMAND_KEY, RESPONSE_HEADER_ETAG_KEY};
use serde_json::Value;
use tracing::instrument;

pub(crate) mod keys;
pub(crate) mod log;
pub(crate) mod methods;
pub(crate) mod persistence;
pub(crate) mod url;

#[derive(Clone)]
pub(crate) struct ProcessingChain {
    links: Vec<Arc<dyn ChainLink>>,
}

impl ProcessingChain {
    pub(crate) fn new(links: Vec<Arc<dyn ChainLink>>) -> Self {
        Self { links }
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

        let body = match body {
            Some(Either::Left(body)) => body.into_inner(),
            Some(Either::Right(body)) => body.into_inner(),
            None => Value::Null,
        };

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
                    return HttpResponse::InternalServerError()
                        .body("No response data found".to_string());
                };

                let response_data: serde_json::Value = serde_json::from_str(response_data).unwrap();
                let mut builder = HttpResponse::Ok();

                if let Some(etag) = context.get(RESPONSE_HEADER_ETAG_KEY) {
                    builder = builder.append_header(("ETag", etag.to_string())).append_header(("Cache-Control", "private, max-age=30")).take();
                }

                return builder.json(response_data);
            }

            let mut builder = HttpResponse::Accepted();

            if let Some(command_header) = context.get(RESPONSE_HEADER_COMMAND_KEY) {
                builder = builder.append_header(("X-Command-Id", command_header.to_string())).take();
            }

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

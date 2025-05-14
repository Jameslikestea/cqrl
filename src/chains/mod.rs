use std::{future::Future, pin::Pin, sync::Arc};

use actix_web::{Handler, HttpRequest, HttpResponse};
use async_trait::async_trait;
use contexts::ContextManager;

pub(crate) mod keys;
pub(crate) mod methods;
pub(crate) mod persistence;
pub(crate) mod url;

#[derive(Clone)]
pub(crate) struct ProcessingChain {
    links: Vec<Arc<dyn ChainLink>>
}

impl ProcessingChain {
    pub(crate) fn new(links: Vec<Arc<dyn ChainLink>>) -> Self {
        Self{
            links
        }
    }
}

impl Handler<HttpRequest> for ProcessingChain {
    type Output = HttpResponse;
    type Future = Pin<Box<dyn Future<Output = HttpResponse>>>;

    fn call(&self, request: HttpRequest) -> Self::Future {
        let links = self.links.clone();
        
        Box::pin(async move {
            let mut context = ContextManager::new();
    
            for link in links.iter() {
                context = match link.process(&context, &request).await {
                    Ok(context) => context,
                    Err(e) => {
                        return HttpResponse::InternalServerError().body(e.to_string());
                    }
                };
            }

            if context.get("response_data").is_some() {
                let Some(response_data) = context.get("response_data") else {
                    return HttpResponse::InternalServerError().body("No response data found".to_string());
                };

                let response_data: serde_json::Value = serde_json::from_str(response_data).unwrap();
                return HttpResponse::Ok().json(response_data);
            }

            HttpResponse::Accepted().body("")
        })
    }
}
#[async_trait(?Send)]
pub(crate) trait ChainLink: Send + Sync {
    async fn process(&self, context: &ContextManager<String, String>, _request: &HttpRequest) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        Ok(context.clone())
    }
}

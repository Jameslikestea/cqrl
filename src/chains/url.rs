use std::{collections::HashMap, sync::Arc};

use actix_web::{
    Either, FromRequest,
    web::{Form, Json, Query},
};
use async_trait::async_trait;
use contexts::ContextManager;
use serde::Deserialize;
use serde_json::Value;
use tracing::instrument;

use super::{
    ChainLink,
    keys::{URL_QUERY_ID_KEY, URL_QUERY_PAGE_KEY, URL_QUERY_PAGE_SIZE_KEY},
};

pub(crate) struct URLChain;

#[derive(Clone, Deserialize)]
struct URLQuery {
    id: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[async_trait(?Send)]
impl ChainLink for URLChain {
    #[instrument(skip(self, context, request, _body) name = "url_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        request: &actix_web::HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut local_context = HashMap::new();

        let query = Query::<URLQuery>::extract(request).await.unwrap();
        let inner = query.into_inner();

        match inner.id {
            Some(id) => {
                local_context.insert(URL_QUERY_ID_KEY.to_string(), id);
            }
            None => (),
        }

        match inner.page {
            Some(page) => {
                local_context.insert(URL_QUERY_PAGE_KEY.to_string(), page.to_string());
            }
            None => {
                local_context.insert(URL_QUERY_PAGE_KEY.to_string(), "0".to_string());
            }
        }

        match inner.page_size {
            Some(page_size) => {
                local_context.insert(URL_QUERY_PAGE_SIZE_KEY.to_string(), page_size.to_string());
            }
            None => {
                local_context.insert(URL_QUERY_PAGE_SIZE_KEY.to_string(), "10".to_string());
            }
        }

        context.push(local_context);

        Ok(context)
    }
}

use std::{collections::HashMap, sync::Arc};

use actix_web::{web::{Path}, FromRequest, HttpRequest};
use contexts::ContextManager;
use async_trait::async_trait;
use parser::API;

use super::{keys::{METHOD_KEY, METHOD_TYPE_KEY, METHOD_TYPE_MUTATION, METHOD_TYPE_QUERY}, ChainLink};

pub(crate) struct QueryMethod {
    api: Arc<API>
}

impl QueryMethod {
    pub(crate) fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

#[async_trait(?Send)]
impl ChainLink for QueryMethod {
    async fn process(&self, context: &ContextManager<String, String>, request: &HttpRequest) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut local_context = HashMap::new();

        let path = Path::<(String,)>::extract(request).await.unwrap();
        let (method,) = path.into_inner();

        local_context.insert(METHOD_KEY.to_string(), method.clone());
        local_context.insert(METHOD_TYPE_KEY.to_string(), METHOD_TYPE_QUERY.to_string());

        if !self.api.queries.iter().any(|q| q.name == method) {
            return Err("Method not found".into());
        }

        context.push(local_context);

        Ok(context)
    }
}

pub(crate) struct CommandMethod {
    api: Arc<API>
}

impl CommandMethod {
    pub(crate) fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

#[async_trait(?Send)]
impl ChainLink for CommandMethod {
    async fn process(&self, context: &ContextManager<String, String>, request: &HttpRequest) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut local_context = HashMap::new();

        let path = Path::<(String,)>::extract(request).await.unwrap();
        let (method,) = path.into_inner();

        local_context.insert(METHOD_KEY.to_string(), method.clone());
        local_context.insert(METHOD_TYPE_KEY.to_string(), METHOD_TYPE_MUTATION.to_string());

        if !self.api.commands.iter().any(|c| c.name == method) {
            return Err("Method not found".into());
        }

        context.push(local_context);

        Ok(context)
    }
}

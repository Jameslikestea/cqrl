use std::{collections::HashMap, sync::Arc};

use actix_web::{
    FromRequest, HttpRequest,
    web::Path,
};
use async_trait::async_trait;
use contexts::ContextManager;
use parser::{API, DataTypes, Model};
use serde_json::{Map, Value};
use tracing::instrument;

use super::{
    ChainLink,
    keys::{METHOD_KEY, METHOD_TYPE_KEY, METHOD_TYPE_MUTATION, METHOD_TYPE_QUERY, COMMAND_BODY_KEY},
};

pub(crate) struct QueryMethod {
    api: Arc<API>,
}

impl QueryMethod {
    pub(crate) fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

#[async_trait(?Send)]
impl ChainLink for QueryMethod {
    #[instrument(skip(self, context, request, _body) name = "query_method_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
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
    api: Arc<API>,
}

impl CommandMethod {
    pub(crate) fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

impl CommandMethod {
    #[instrument(skip(self, model) name = "validate_body")]
    fn validate_body(&self, model: &Model, body: &Value) -> Result<String, Box<dyn std::error::Error>> {
        let body = body.clone();

        if !body.is_object() {
            return Err("Invalid body".into());
        }

        tracing::info!("validating body for model: {}", model.name);

        let inner = body.as_object().unwrap();

        let mut object: Map<String, Value> = Map::new();

        for field in model.properties.iter() {
            match inner.get(field.name.as_str()) {
                Some(value) => match &field.datatype {
                    DataTypes::ID => {
                        if !value.is_string() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::String => {
                        if !value.is_string() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::Number => {
                        if !value.is_number() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::Datetime => {
                        if !value.is_string() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::Boolean => {
                        if !value.is_boolean() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::Pattern(re) => {
                        if !value.is_string()
                            || !regex::Regex::new(&re)
                                .unwrap()
                                .is_match(value.as_str().unwrap())
                        {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                    DataTypes::Model(_) => {
                        if !value.is_string() {
                            return Err("Invalid field type".into());
                        }

                        object.insert(field.name.clone(), value.clone());
                    }
                },
                None => {
                    if field.required {
                        return Err("Missing required field".into());
                    }
                }
            }
        }

        let object = Value::Object(object);
        let body = serde_json::to_string(&object)?;

        Ok(body)
    }
}

#[async_trait(?Send)]
impl ChainLink for CommandMethod {
    #[instrument(skip(self, context, request, _body) name = "command_method_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut local_context = HashMap::new();

        let path = Path::<(String,)>::extract(request).await.unwrap();
        let (method,) = path.into_inner();

        local_context.insert(METHOD_KEY.to_string(), method.clone());
        local_context.insert(
            METHOD_TYPE_KEY.to_string(),
            METHOD_TYPE_MUTATION.to_string(),
        );

        match self.api.commands.iter().find(|c| c.name == method) {
            Some(command) => {
                let model = match self
                    .api
                    .models
                    .iter()
                    .find(|m| m.name == command.modelled_by)
                {
                    Some(model) => model,
                    None => return Err("Model not found".into()),
                };

                match self.validate_body(model, _body) {
                    Ok(body) => {
                        local_context.insert(COMMAND_BODY_KEY.to_string(), body);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                };
            }
            None => {
                return Err("Method not found".into());
            }
        }

        context.push(local_context);

        Ok(context)
    }
}

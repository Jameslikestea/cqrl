use std::{collections::HashMap, sync::Arc};

use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use futures::StreamExt;
use mongodb::{
    bson::{self, doc, Bson, DateTime, Document},
    Client,
};
use parser::API;
use serde_json::Value;
use tracing::instrument;

use crate::chains::keys::{
    AUTH_CONTEXT_ID_KEY, AUTH_CONTEXT_TYPE_KEY, RESPONSE_HEADER_COMMAND_KEY,
    RESPONSE_HEADER_OBJECT_COUNT_KEY, URL_QUERY_ID_KEY, URL_QUERY_PAGE_KEY,
    URL_QUERY_PAGE_SIZE_KEY,
};

use super::{
    keys::{COMMAND_BODY_KEY, METHOD_KEY, RESPONSE_DATA_KEY, RESPONSE_HEADER_ETAG_KEY},
    ChainLink,
};

pub(crate) struct MongoQueryChain {
    _api: Arc<API>,
    _store: Arc<Client>,
}

impl MongoQueryChain {
    pub(crate) fn new(_api: Arc<API>, _store: Arc<mongodb::Client>) -> Self {
        Self { _api, _store }
    }
}

#[async_trait(?Send)]
impl ChainLink for MongoQueryChain {
    #[instrument(skip(self, context, _request, _body) name = "mongo_query_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: Arc<HttpRequest>,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut hm = HashMap::new();

        let mut agg_pipeline = vec![];

        let mut match_query = bson::Document::new();
        let mut projection = doc! {};
        let mut single = false;

        if let Some(object_id) = context.get(URL_QUERY_ID_KEY) {
            match_query.insert("_id", Bson::String(object_id.to_string()));
            single = true;
        };

        if let Some(method) = context.get(METHOD_KEY) {
            match_query.insert("metadata.type", Bson::String(method.to_string()));

            let Some(query_method) = self._api.queries.iter().find(|q| q.name == *method) else {
                return Err("Method not found".into());
            };

            let Some(model) = self
                ._api
                .models
                .iter()
                .find(|m| m.name == query_method.modelled_by)
            else {
                return Err("Model not found".into());
            };

            projection = Document::new();
            for field in model.properties.iter() {
                if field.primary {
                    projection.insert(field.name.clone(), "$_id");
                } else {
                    projection.insert(field.name.clone(), format!("$data.{}", field.name));
                }
            }
            tracing::info!(
                method = method,
                model = model.name,
                "projecting in return type"
            );
        }

        agg_pipeline.push(doc! {
            "$match": match_query.clone(),
        });
        agg_pipeline.push(doc! {
            "$project": projection,
        });

        match (
            context.get(URL_QUERY_PAGE_KEY),
            context.get(URL_QUERY_PAGE_SIZE_KEY),
        ) {
            (Some(page), Some(page_size)) => {
                let u32_page = page.parse::<u32>();
                let u32_page_size = page_size.parse::<u32>();

                if u32_page.is_err() || u32_page_size.is_err() {
                    return Err("Invalid page or page size".into());
                }

                let u32_page = u32_page.unwrap();
                let mut u32_page_size = u32_page_size.unwrap();

                if u32_page_size > 100 {
                    u32_page_size = 100; // TODO: make this configurable
                }

                let skip = u32_page * u32_page_size;
                let limit = u32_page_size;

                agg_pipeline.push(doc! {
                    "$skip": skip
                });

                agg_pipeline.push(doc! {
                    "$limit": limit
                });
            }
            (Some(page), None) => {
                let u32_page = page.parse::<u32>();

                if u32_page.is_err() {
                    return Err("Invalid page".into());
                }

                let skip = u32_page.unwrap() * 10;
                let limit = 10;

                agg_pipeline.push(doc! {
                    "$skip": skip
                });

                agg_pipeline.push(doc! {
                    "$limit": limit
                });
            }
            (None, Some(page_size)) => {
                let u32_page_size = page_size.parse::<u32>();

                if u32_page_size.is_err() {
                    return Err("Invalid page size".into());
                }

                let mut limit = u32_page_size.unwrap();
                if limit > 100 {
                    limit = 100; // TODO: make this configurable
                }

                agg_pipeline.push(doc! {
                    "$limit": limit
                });
            }
            (None, None) => {
                agg_pipeline.push(doc! {
                    "$limit": 10
                });
            }
        }

        let mut result = self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .aggregate(agg_pipeline)
            .await
            .unwrap();

        let count = self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .count_documents(match_query.clone())
            .await
            .unwrap();

        let mut objects = Vec::new();

        while let Some(operation) = result.next().await {
            let operation = operation.unwrap();
            objects.push(operation);
        }

        let response = if single && objects.len() > 0 {
            serde_json::to_string(&objects[0]).unwrap()
        } else {
            serde_json::to_string(&objects).unwrap()
        };

        let etag = crc64::crc64(0, response.as_bytes());

        hm.insert(RESPONSE_DATA_KEY.to_string(), response);
        hm.insert(
            RESPONSE_HEADER_OBJECT_COUNT_KEY.to_string(),
            count.to_string(),
        );
        hm.insert(
            RESPONSE_HEADER_ETAG_KEY.to_string(),
            format!("\"{}\"", etag),
        );
        context.push(hm);

        Ok(context)
    }
}

pub(crate) struct MongoCommandChain {
    _api: Arc<API>,
    _store: Arc<Client>,
}

impl MongoCommandChain {
    pub(crate) fn new(_api: Arc<API>, _store: Arc<mongodb::Client>) -> Self {
        Self { _api, _store }
    }
}

#[async_trait(?Send)]
impl ChainLink for MongoCommandChain {
    #[instrument(skip(self, context, _request, _body) name = "mongo_command_chain")]
    async fn process(
        &self,
        context: &ContextManager<String, String>,
        _request: Arc<HttpRequest>,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut hm = HashMap::new();
        let id = ulid::Ulid::new().to_string();

        let Some(command_body) = context.get(COMMAND_BODY_KEY) else {
            return Err("Command body not found".into());
        };

        let Some(method) = context.get(METHOD_KEY) else {
            return Err("Method not found".into());
        };

        let json = serde_json::from_str::<Value>(command_body)?;

        let mut doc = Document::new();

        let mut auth_context = Document::new();
        auth_context.insert(
            "authtype",
            Bson::String(match context.get(AUTH_CONTEXT_TYPE_KEY) {
                Some(authtype) => authtype.to_string(),
                None => "unauthenticated".to_string(),
            }),
        );

        match context.get(AUTH_CONTEXT_ID_KEY) {
            Some(auth_id) => {
                auth_context.insert("authid", Bson::String(auth_id.to_string()));
            }
            None => {}
        }

        let mut metadata = Document::new();
        metadata.insert("type", Bson::String(method.to_string()));
        metadata.insert("created_at", Bson::DateTime(DateTime::now()));
        metadata.insert("authcontext", Bson::Document(auth_context));

        match context.get(URL_QUERY_ID_KEY) {
            Some(object_id) => {
                metadata.insert("subject_id", Bson::String(object_id.to_string()));
            }
            None => {}
        }

        let mut data = Document::new();

        for (key, value) in json.as_object().unwrap().iter() {
            data.insert(key, bson::to_bson(value).unwrap());
        }

        doc.insert("_id", Bson::String(id.clone()));
        doc.insert("metadata", Bson::Document(metadata));
        doc.insert("data", Bson::Document(data));

        self._store
            .database("cqrl")
            .collection("operations")
            .insert_one(Bson::Document(doc))
            .await?;

        hm.insert(RESPONSE_HEADER_COMMAND_KEY.to_string(), id.clone());
        context.push(hm);

        tracing::info!(command_body = json.to_string(), "command body");

        Ok(context.clone())
    }
}

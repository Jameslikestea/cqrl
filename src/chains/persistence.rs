use std::{collections::HashMap, sync::Arc};

use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use futures::StreamExt;
use mongodb::{
    Client,
    bson::{Document, doc},
};
use parser::API;
use serde_json::Value;
use tracing::instrument;

use super::{
    ChainLink,
    keys::{METHOD_KEY, RESPONSE_DATA_KEY},
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
        _request: &HttpRequest,
        _body: &Value,
    ) -> Result<ContextManager<String, String>, Box<dyn std::error::Error>> {
        let mut context = context.clone();
        let mut hm = HashMap::new();

        let mut agg_pipeline = vec![];

        let mut match_query = doc! {
            "$match": {},
        };
        let mut projection = doc! {};

        if let Some(method) = context.get(METHOD_KEY) {
            match_query = doc! {
                "$match": {
                    "metadata.type": method,
                },
            };

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

        agg_pipeline.push(match_query);
        agg_pipeline.push(doc! {
            "$project": projection,
        });

        let mut result = self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .aggregate(agg_pipeline)
            .await
            .unwrap();

        let mut objects = Vec::new();

        while let Some(operation) = result.next().await {
            let operation = operation.unwrap();
            objects.push(operation);
        }

        hm.insert(
            RESPONSE_DATA_KEY.to_string(),
            serde_json::to_string(&objects).unwrap(),
        );
        context.push(hm);

        Ok(context)
    }
}

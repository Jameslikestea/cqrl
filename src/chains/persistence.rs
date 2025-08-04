use std::{collections::HashMap, sync::Arc};

use actix_web::HttpRequest;
use async_trait::async_trait;
use contexts::ContextManager;
use errors::CQRLError;
use futures::StreamExt;
use mongodb::{
    bson::{self, doc, Bson, DateTime, Document},
    Client,
};
use parser::{Query, API};
use serde_json::Value;
use tracing::{info, instrument};

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

    pub(crate) fn build_aggregate_match_query(
        &self,
        context: &ContextManager<String, String>,
    ) -> Option<bson::Document> {
        match self.build_match_query(context) {
            Some(match_query) => Some(doc! {
                "$match": match_query,
            }),
            None => None,
        }
    }

    pub(crate) fn get_query(&self, context: &ContextManager<String, String>) -> Option<Query> {
        let Some(method) = context.get(METHOD_KEY) else {
            return None;
        };

        let Some(query) = self._api.queries.iter().find(|q| q.name == *method) else {
            return None;
        };

        Some(query.clone())
    }

    pub(crate) fn build_match_query(
        &self,
        context: &ContextManager<String, String>,
    ) -> Option<bson::Document> {
        let mut match_query = bson::Document::new();

        if let Some(object_id) = context.get(URL_QUERY_ID_KEY) {
            match_query.insert("_id", Bson::String(object_id.to_string()));
        }

        if let Some(method) = context.get(METHOD_KEY) {
            match_query.insert("metadata.type", Bson::String(method.to_string()));
        }

        if let Some(query) = self.get_query(context) {
            if !query.public {
                if let Some(auth_id) = context.get(AUTH_CONTEXT_ID_KEY) {
                    match_query.insert(
                        "metadata.authcontext.read",
                        Bson::String(auth_id.to_string()),
                    );
                } else {
                    // This is a private query, so this should never match any documents. If it does,
                    // it means that the DBA has manually updated the document. It is intentional that this
                    // functionality exists, to enable the edge case that something that is usually private
                    // can be made public for unauthenticated users.
                    match_query.insert("metadata.authcontext.unauthenticated", Bson::Boolean(true));
                }
            }
        }

        Some(match_query)
    }

    pub(crate) fn build_projection(
        &self,
        context: &ContextManager<String, String>,
    ) -> Option<bson::Document> {
        let mut projection = doc! {};

        let query_model = match self.get_query(context) {
            Some(query_method) => query_method,
            None => return Some(doc! {}),
        };

        let model = match self
            ._api
            .models
            .iter()
            .find(|m| m.name == query_model.modelled_by)
        {
            Some(model) => model,
            None => return Some(doc! {}),
        };

        for field in model.properties.iter() {
            if field.primary {
                projection.insert(field.name.clone(), "$_id");
            } else {
                projection.insert(field.name.clone(), format!("$data.{}", field.name));
            }
        }

        Some(doc! {
            "$project": projection,
        })
    }

    pub(crate) fn build_skip(
        &self,
        context: &ContextManager<String, String>,
    ) -> Option<bson::Document> {
        let page = self.get_page(context);
        let page_size = self.get_page_size(context);

        Some(doc! {
            "$skip": page * page_size,
        })
    }

    pub(crate) fn build_limit(
        &self,
        context: &ContextManager<String, String>,
    ) -> Option<bson::Document> {
        let page_size = self.get_page_size(context);

        Some(doc! {
            "$limit": page_size,
        })
    }

    pub(crate) fn id_present(&self, context: &ContextManager<String, String>) -> bool {
        context.get(URL_QUERY_ID_KEY).is_some()
    }

    pub(crate) fn get_page(&self, context: &ContextManager<String, String>) -> u32 {
        let Some(page) = context.get(URL_QUERY_PAGE_KEY) else {
            return 0;
        };

        let u32_page = page.parse::<u32>();

        if u32_page.is_err() {
            return 0;
        }

        u32_page.unwrap()
    }

    pub(crate) fn get_page_size(&self, context: &ContextManager<String, String>) -> u32 {
        let Some(page_size) = context.get(URL_QUERY_PAGE_SIZE_KEY) else {
            return 50;
        };

        let u32_page_size = page_size.parse::<u32>();

        if u32_page_size.is_err() {
            return 50;
        }

        let mut u32_page_size = u32_page_size.unwrap();

        if u32_page_size > 100 {
            u32_page_size = 100; // TODO: make this configurable
        }

        u32_page_size
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

        let agg_pipeline = vec![
            self.build_aggregate_match_query(&context),
            self.build_projection(&context),
            self.build_skip(&context),
            self.build_limit(&context),
        ];

        let mut result = self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .aggregate(
                agg_pipeline
                    .iter()
                    .filter_map(|x| x.clone())
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap();

        let count = self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .count_documents(self.build_match_query(&context).unwrap_or_default())
            .await
            .unwrap();

        let mut objects = Vec::new();

        while let Some(operation) = result.next().await {
            let operation = operation.unwrap();
            objects.push(operation);
        }

        let response = if self.id_present(&context) && objects.len() > 0 {
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

    pub(crate) async fn check_permission(&self, context: &ContextManager<String, String>) -> bool {
        info!("Checking permissions");
        // We can assume if the id is not present then the user can take the action.
        let Some(id) = context.get(URL_QUERY_ID_KEY) else {
            return true;
        };

        let Some(user_id) = context.get(AUTH_CONTEXT_ID_KEY) else {
            return false;
        };

        let Some(method) = context.get(METHOD_KEY) else {
            return false;
        };

        let Some(command) = self._api.commands.iter().find(|c| c.name == *method) else {
            return false;
        };

        let Some(query) = self
            ._api
            .queries
            .iter()
            .find(|m| m.name == command.authorized_by.clone())
        else {
            return false;
        };

        match self
            ._store
            .database("cqrl")
            .collection::<Value>("objects")
            .count_documents(doc! {
                "_id": id,
                "metadata.type": query.name.clone(),
                "metadata.authcontext.write": user_id,
            })
            .await
        {
            Ok(count) => count > 0,
            Err(_) => false,
        }
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

        if !self.check_permission(&context).await {
            return Err(CQRLError::PermissionDenied.into());
        };

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

use std::sync::Arc;

use super::{PersistenceObject, Store};
use cloudevents::{AttributesReader, Data, Event};
use errors::CQRLResult;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use mongodb::{
    bson::{self, doc, Bson},
    Client, IndexModel,
};
use parser::API;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task;

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
    api: Arc<API>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoObject {
    #[serde(rename = "_id")]
    id: String,
    metadata: Value,
    data: Value,
}

impl Into<PersistenceObject> for MongoObject {
    fn into(self) -> PersistenceObject {
        PersistenceObject {
            id: self.id,
            metadata: self.metadata,
            data: self.data,
        }
    }
}

impl From<PersistenceObject> for MongoObject {
    fn from(object: PersistenceObject) -> Self {
        Self {
            id: object.id,
            metadata: object.metadata,
            data: object.data,
        }
    }
}

impl MongoStore {
    pub fn new(client: Client, api: Arc<API>) -> Self {
        Self { client, api }
    }

    pub async fn init(&mut self) -> CQRLResult<()> {
        self.client
            .database("cqrl")
            .collection::<MongoObject>("operations")
            .create_index(
                IndexModel::builder()
                    .keys(doc! {"metadata.type": 1})
                    .build(),
            )
            .await
            .unwrap();
        Ok(())
    }
}

impl MongoStore {
    fn event_to_bson(self: &Self, evt: Event) -> Bson {
        println!("Converting event to bson: {:?}", evt.id());
        let data = evt.data().unwrap();
        let value = match data {
            Data::Json(json) => json.clone(),
            Data::String(string) => serde_json::from_str(&string).unwrap(),
            Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
        };

        let mut document = bson::Document::new();
        document.insert("metadata.type", evt.ty());
        document.insert("metadata.time", evt.time().unwrap().to_rfc3339());

        let api = self.api.clone();
        let command = api
            .queries
            .iter()
            .find(|command| command.name == evt.ty())
            .unwrap();
        let model = api
            .models
            .iter()
            .find(|model| model.name == command.modelled_by)
            .unwrap();

        for property in model.properties.iter() {
            let value = value.get(property.name.clone());
            if value.is_some() {
                document.insert(
                    format!("data.{}", property.name.clone()),
                    bson::to_bson(&value.unwrap()).unwrap(),
                );
            }
        }

        Bson::Document(document)
    }
}

impl Store for MongoStore {
    async fn store_operation(
        &mut self,
        _key: String,
        _value: Value,
        _operation_type: String,
    ) -> CQRLResult<PersistenceObject> {
        let mongo_object = MongoObject {
            id: ulid::Ulid::new().to_string(),
            metadata: json!({
                "type": _operation_type,
            }),
            data: _value,
        };
        self.client
            .database("cqrl")
            .collection("operations")
            .insert_one(mongo_object.clone())
            .await
            .unwrap();
        Ok(mongo_object.into())
    }

    async fn get_object(
        &self,
        id: Option<String>,
        object_type: String,
    ) -> CQRLResult<serde_json::Value> {
        let query = match id {
            Some(id) => doc! {
                "metadata.type": object_type,
                "_id": id,
            },
            None => doc! {
                "metadata.type": object_type,
            },
        };

        let mut object = self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .find(query)
            .limit(100)
            .skip(0)
            .await
            .unwrap();
        let mut objects = Vec::new();
        while let Some(obj) = object.next().await {
            objects.push(obj.unwrap().data);
        }

        Ok(Value::Array(objects))
    }

    async fn store_object(&mut self, _k: String, evt: Event) -> errors::CQRLResult<()> {
        let ty = evt.ty().split(".").last().unwrap().to_string();

        let (_id, query) = match evt.subject() {
            Some(subject) => (
                subject.to_string(),
                doc! {
                    "metadata.type": ty.clone(),
                    // Check that we've not already processed this event for this subject
                    "metadata.lineage": {
                        "$nin": vec![evt.id()],
                    },
                    "_id": subject.to_string(),
                },
            ),
            None => {
                println!("No subject found for event: {:?}", evt.id());
                return Err(errors::CQRLError::Generic);
            }
        };

        let update = doc! {
            "$setOnInsert": {
                "_id": _id,
            },
            "$addToSet": {
                "metadata.lineage": evt.id(),
            },
            "$set": self.event_to_bson(evt.clone()),
        };

        match self
            .client
            .database("cqrl")
            .collection::<MongoObject>("objects")
            .update_one(query, update)
            .upsert(true)
            .await
        {
            Ok(result) => {
                println!(
                    "Applied event: {:?}. Modified: {:?}",
                    evt.id(),
                    result.modified_count
                );
                Ok(())
            }
            Err(err) => {
                println!("Error storing object: {:?}", err);
                Err(errors::CQRLError::StoreError {
                    error: format!("Error storing object: {:?}", err),
                })
            }
        }
    }

    fn watch_operation(
        &mut self,
    ) -> impl futures::StreamExt<Item = super::PersistenceObject> + Send {
        let (mut _sender, receiver) = mpsc::unbounded();

        let client = self.client.clone();
        task::spawn(async move {
            let mut stream = client
                .database("cqrl")
                .collection::<MongoObject>("operations")
                .watch()
                .await
                .unwrap();
            while let Some(response) = stream.next().await {
                if let Ok(object) = response {
                    match _sender.send(object.full_document.unwrap().into()).await {
                        Ok(_) => (),
                        Err(err) => {
                            println!("Error sending object: {:?}", err);
                        }
                    }
                }
            }
        });

        receiver
    }
}

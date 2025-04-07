use errors::CQRLResult;
use futures::channel::mpsc;
use mongodb::{bson::{self, doc}, Client, IndexModel};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use surrealdb::RecordId;
use tokio::task;
use futures::{StreamExt, SinkExt};
use crate::{Store, PersistenceObject};

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoObject {
    #[serde(rename = "_id")]
    id: String,
    metadata: Value,
    data: Value,
}

impl Into<PersistenceObject> for MongoObject{
    fn into(self) -> PersistenceObject {
        PersistenceObject {
            id: RecordId::from(("operation", self.id)),
            metadata: self.metadata,
            data: self.data,
        }
    }
}

impl MongoStore {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn init(&mut self) -> CQRLResult<()> {
        self.client.database("cqrl").collection::<MongoObject>("operations").create_index(IndexModel::builder().keys(doc!{"metadata.type": 1}).build()).await.unwrap();
        Ok(())
    }
}

impl Store for MongoStore {
    async fn store_operation(&mut self, _key: String, _value: Value, _operation_type: String) -> CQRLResult<PersistenceObject> {
        let mongo_object = MongoObject {
            id: ulid::Ulid::new().to_string(),
            metadata: json!({
                "type": _operation_type,
            }),
            data: _value,
        };
        self.client.database("cqrl").collection("operations").insert_one(mongo_object.clone()).await.unwrap();
        Ok(mongo_object.into())
    }
    
    async fn get_object(&self, _id: Option<String>, _object_type: String) -> CQRLResult<serde_json::Value> {
        Ok(Value::Null)
    }
    
    async fn store_object(&mut self, _k: String, _v: serde_json::Value, _object_type: String) -> errors::CQRLResult<()> {
        let id = ulid::Ulid::new().to_string();

        let query = doc!{
            "metadata.id": _k,
        };

        let update = doc!{
            "$setOnInsert": {
                "_id": id,
            },
            "$set": bson::to_bson(&_v).unwrap(),
        };

        self.client.database("cqrl").collection::<MongoObject>("objects").update_one(query, update).upsert(true).await.unwrap();
        println!("Stored object: {:?}", _v);
        Ok(())
    }
    
    fn watch_operation(&mut self) -> impl futures::StreamExt<Item = crate::PersistenceObject> + Send {
        let (mut _sender, receiver) = mpsc::unbounded();

        let client = self.client.clone();
        task::spawn(async move {
            let mut stream = client.database("cqrl").collection::<MongoObject>("operations").watch().await.unwrap();
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

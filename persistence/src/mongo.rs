use cloudevents::{AttributesReader, Data, Event};
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

impl From<PersistenceObject> for MongoObject {
    fn from(object: PersistenceObject) -> Self {
        Self { id: object.id.key().to_string(), metadata: object.metadata, data: object.data }
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
    
    async fn get_object(&self, id: Option<String>, object_type: String) -> CQRLResult<serde_json::Value> {
        let query = match id {
            Some(id) => doc!{
                "metadata.type": object_type,
                "_id": id,
            },
            None => doc!{
                "metadata.type": object_type,
            }
        };

        let mut object = self.client.database("cqrl").collection::<MongoObject>("objects").find(query).limit(100).skip(0).await.unwrap();
        let mut objects = Vec::new();
        while let Some(obj) = object.next().await {
            objects.push(obj.unwrap().data);
        }

        Ok(Value::Array(objects))
    }
    
    async fn store_object(&mut self, _k: String, evt: Event) -> errors::CQRLResult<()> {
        let id = ulid::Ulid::new().to_string();
        let ty = evt.ty().split(".").last().unwrap().to_string();
        
        let (_id, query) = match evt.subject() {
            Some(subject) => (subject.to_string(), doc!{
                "metadata.type": ty.clone(),
                // Check that we've not already processed this event for this subject
                "metadata.lineage": {
                    "$nin": vec![evt.id()],
                },
                "_id": subject.to_string(),
            }),
            None => {
                println!("No subject found for event: {:?}", evt.id());
                return Err(errors::CQRLError::Generic);
            },
        };

        let data = match evt.data() {
            Some(data) => match data {
                Data::Json(json) => json.clone(),
                Data::String(string) => serde_json::from_str(&string).unwrap(),
                Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
            },
            None => serde_json::from_str("").unwrap(),
        };


        let update = doc!{
            "$setOnInsert": {
                "_id": _id,
            },
            "$addToSet": {
                "metadata.lineage": evt.id(),
            },
            "$set": {
                "metadata.type": ty.clone(),
                "metadata.time": evt.time().unwrap().to_rfc3339(),
                "data": bson::to_bson(&data).unwrap(),
            },
        };

        match self.client.database("cqrl").collection::<MongoObject>("objects").update_one(query, update).upsert(true).await {
            Ok(result) => {
                println!("Stored object: {:?}. Modified: {:?}", evt, result.modified_count);
                Ok(())
            },
            Err(err) => {
                println!("Error storing object: {:?}", err);
                Err(errors::CQRLError::StoreError)
            }
        }
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

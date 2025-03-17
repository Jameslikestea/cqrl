use errors::CQRLResult;
use futures::channel::mpsc;
use mongodb::{bson::document, Client};
use serde_json::{json, Value};
use surrealdb::RecordId;
use tokio::task;

use crate::{Store, SurrealObject};

#[derive(Clone)]
pub struct MongoStore {
    client: Client,
}

impl MongoStore {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl Store for MongoStore {
    async fn store_operation(&mut self, _key: String, _value: Value, _operation_type: String) -> CQRLResult<SurrealObject> {
        self.client.database("cqrl").collection("operations").insert_one(json!({
            "_id": format!("{}:{}", _operation_type, ulid::Ulid::new().to_string()),
            "metadata": Value::Null,
            "data": _value,
        })).await.unwrap();
        let id = RecordId::from(("operation", ulid::Ulid::new().to_string()));
        Ok(SurrealObject {
            id: id,
            metadata: Value::Null,
            data: _value,
        })
    }
    
    async fn get_object(&self, id: Option<String>, object_type: String) -> CQRLResult<serde_json::Value> {
        Ok(Value::Null)
    }
    
    async fn store_object(&mut self, k: String, v: serde_json::Value, object_type: String) -> errors::CQRLResult<()> {
        Ok(())
    }
    
    fn watch_operation(&mut self) -> impl futures::StreamExt<Item = crate::SurrealObject> + Send {
        let (mut sender, receiver) = mpsc::unbounded();

        task::spawn(async move {
            // let mut stream: surrealdb::method::Stream<Vec<SurrealObject>> = db_clone.select("command").live().await.unwrap();
            // while let Some(response) = stream.next().await {
            //     if let Ok(object) = response {
            //         match sender.send(object.data).await {
            //             Ok(_) => (),
            //             Err(err) => {
            //                 println!("Error sending object: {:?}", err);
            //             },
            //         };
            //     }
            // }
        });

        receiver
    }
}

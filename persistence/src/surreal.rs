use errors::{CQRLError, CQRLResult};
use futures::{SinkExt, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use surrealdb::{Connection, RecordId, Surreal};
use futures::stream::StreamExt;
use futures::channel::mpsc;
use tokio::task;

use crate::{Permission, PermissionStore, Store, PersistenceObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    id: RecordId,
}

#[derive(Clone)]
pub struct SurrealStore<S> where S: Connection {
    db: Surreal<S>,
}

impl<S> SurrealStore<S> where S: Connection {
    pub fn new(db: Surreal<S>) -> Self {
        Self { db }
    }

    pub async fn init(&mut self) -> CQRLResult<()> {
        self.db.query("DEFINE INDEX OVERWRITE object_type ON object FIELDS metadata.object_type;").await.unwrap();
        self.db.query("DEFINE INDEX OVERWRITE command_type ON command FIELDS metadata.command_type;").await.unwrap();
        self.db.query("DEFINE TABLE OVERWRITE object SCHEMALESS CHANGEFEED 1h;").await.unwrap();
        self.db.query("DEFINE TABLE OVERWRITE command SCHEMALESS CHANGEFEED 1h;").await.unwrap();
        Ok(())
    }
}

impl<S> Store for SurrealStore<S> where S: Connection {
    async fn store_operation(&mut self, _key: String, _value: Value, _operation_type: String) -> CQRLResult<crate::PersistenceObject> {
        let result = self.db
            .query("CREATE command:ulid() CONTENT {metadata: {type: $type}, data: $data}")
            .bind(("type", _operation_type))
            .bind(("data", _value))
            .await;

        println!("{:?}", result);

        match result {
            Ok(mut response) => {
                if let Ok(created) = response.take::<Vec<crate::PersistenceObject>>(0) {
                    if let Some(record) = created.first() {
                        return Ok(record.clone());
                    }
                }
                Err(CQRLError::StoreError)
            },
            _ => Err(CQRLError::StoreError)
        }
    }

    async fn get_object(&self, _key: Option<String>, object_type: String) -> CQRLResult<Value> {
        let query = 
        match _key {
            Some(key) => self.db.query("SELECT id, metadata, data FROM object WHERE metadata.object_type = $object_type AND id = $key ORDER BY id DESC;").bind(("object_type", object_type)).bind(("key", RecordId::from(("object", key)))),
            _ => self.db.query("SELECT id, metadata, data FROM object WHERE metadata.object_type = $object_type ORDER BY id DESC;").bind(("object_type", object_type)),
        };
        let mut result = query.await.unwrap();
        let value: Vec<crate::PersistenceObject> = result.take(0 as usize).unwrap();
        println!("{:?}", value);

        Ok(Value::Array(value.iter().map(|v| {
            let mut value = v.data.clone();
            value["__id"] = json!(v.id.key().to_string());

            value
        }).collect()))
    }

    async fn store_object(&mut self, _key: String, _value: Value, _object_type: String) -> CQRLResult<()> {
        Ok(())
    }

    fn watch_operation(&mut self) -> impl Stream<Item = PersistenceObject> + Send {
        let (mut sender, receiver) = mpsc::unbounded();

        let db_clone = self.db.clone();
        task::spawn(async move {
            let mut stream: surrealdb::method::Stream<Vec<PersistenceObject>> = db_clone.select("command").live().await.unwrap();
            while let Some(response) = stream.next().await {
                if let Ok(object) = response {
                    match sender.send(object.data).await {
                        Ok(_) => (),
                        Err(err) => {
                            println!("Error sending object: {:?}", err);
                        },
                    };
                }
            }
        });

        receiver
    }
}

impl<S> PermissionStore for SurrealStore<S> where S: Connection {
    async fn permit(&self, _id: String, _user: String, _permission: Permission) -> CQRLResult<bool> {
        Ok(false)
    }

    async fn grant(&mut self, _id: String, _user: String, _permission: Permission) -> CQRLResult<()> {
        Ok(())
    }

    async fn revoke(&mut self, _id: String, _user: String, _permission: Permission) -> CQRLResult<()> {
        Ok(())
    }
}
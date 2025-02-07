use errors::{CQRLError, CQRLResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use surrealdb::{Connection, RecordId, Surreal};

use crate::{Permission, PermissionStore, Store};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrealObject {
    id: RecordId,
    metadata: Value,
    data: Value,
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
        Ok(())
    }
}

impl<S> Store for SurrealStore<S> where S: Connection {
    async fn store_operation(&mut self, _key: String, _value: Value, _operation_type: String) -> CQRLResult<()> {
        let result = self.db.query("BEGIN").query("CREATE command:ulid() CONTENT {metadata: {type: $operation_type}, data: $command_content};").bind(("operation_type", _operation_type)).bind(("command_content", _value)).query("COMMIT").await;

        println!("{:?}", result);

        match result {
            Ok(_) => Ok(()),
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
        let value: Vec<SurrealObject> = result.take(0 as usize).unwrap();
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
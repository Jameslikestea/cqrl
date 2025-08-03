use cloudevents::{AttributesReader, Data, Event};
use futures::StreamExt;
use std::future::Future;

use errors::CQRLResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceObject {
    pub id: String,
    pub metadata: Value,
    pub data: Value,
}

impl From<cloudevents::Event> for PersistenceObject {
    fn from(event: cloudevents::Event) -> Self {
        Self {
            id: event.id().to_string(),
            metadata: serde_json::json!({
                "type": event.ty(),
                "source": event.source(),
                "time": event.time(),
            }),
            data: match event.data().unwrap() {
                Data::Json(json) => json.clone(),
                Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
                Data::String(string) => serde_json::from_str(&string).unwrap(),
            },
        }
    }
}

pub trait Store: Send + Sync {
    #[allow(dead_code)]
    fn store_operation(
        &mut self,
        k: String,
        v: Value,
        operation_type: String,
    ) -> impl Future<Output = CQRLResult<PersistenceObject>> + Send;
    #[allow(dead_code)]
    fn get_object(
        &self,
        id: Option<String>,
        object_type: String,
    ) -> impl Future<Output = CQRLResult<Value>> + Send;
    fn store_object(&mut self, k: String, v: Event) -> impl Future<Output = CQRLResult<()>> + Send;
    fn watch_operation(&mut self) -> impl StreamExt<Item = PersistenceObject> + Send;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum Permission {
    Read,
    Write,
}

pub trait PermissionStore: Send + Sync {
    fn grant(
        &mut self,
        id: String,
        user: String,
        permission: Permission,
    ) -> impl Future<Output = CQRLResult<()>> + Send;
    fn revoke(
        &mut self,
        id: String,
        user: String,
        permission: Permission,
    ) -> impl Future<Output = CQRLResult<()>> + Send;
}

pub mod memory;
pub mod mongo;

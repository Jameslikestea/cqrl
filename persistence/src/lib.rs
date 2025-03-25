use std::future::Future;
use futures::StreamExt;

use errors::CQRLResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use surreal::SurrealStore;
use surrealdb::{engine::remote::ws::Client, RecordId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceObject {
    pub id: RecordId,
    pub metadata: Value,
    pub data: Value,
}
pub trait Store: Send + Sync {
    fn store_operation(&mut self, k: String, v: Value, operation_type: String) -> impl Future<Output = CQRLResult<PersistenceObject>> + Send;
    fn get_object(&self, id: Option<String>, object_type: String) -> impl Future<Output = CQRLResult<Value>> + Send;
    fn store_object(&mut self, k: String, v: Value, object_type: String) -> impl Future<Output = CQRLResult<()>> + Send;
    fn watch_operation(&mut self) -> impl StreamExt<Item = PersistenceObject> + Send;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum Permission {
    Read,
    Write,
}

pub trait PermissionStore: Send + Sync {
    fn permit(&self, id: String, user: String, permission: Permission) -> impl Future<Output = CQRLResult<bool>> + Send;
    fn grant(&mut self, id: String, user: String, permission: Permission) -> impl Future<Output = CQRLResult<()>> + Send;
    fn revoke(&mut self, id: String, user: String, permission: Permission) -> impl Future<Output = CQRLResult<()>> + Send;
}

pub enum StoreEnum {
    Surreal(SurrealStore<Client>),
}

pub mod memory;
pub mod surreal;
pub mod mongo;
use errors::CQRLResult;
use serde_json::Value;

pub trait Store {
    fn store_operation(&mut self, k: String, v: Value) -> CQRLResult<()>;
    fn get_object(&self, id: Option<String>) -> CQRLResult<Value>;
    fn store_object(&mut self, k: String, v: Value) -> CQRLResult<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]

pub enum Permission {
    Read,
    Write,
}

pub trait PermissionStore {
    fn permit(&self, id: String, user: String, permission: Permission) -> CQRLResult<bool>;
    fn grant(&mut self, id: String, user: String, permission: Permission) -> CQRLResult<()>;
    fn revoke(&mut self, id: String, user: String, permission: Permission) -> CQRLResult<()>;
}

pub mod memory;

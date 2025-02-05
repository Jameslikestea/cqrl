use errors::CQRLResult;
use serde_json::Value;

pub trait Store {
    fn store_operation(&mut self, k: String, v: Value) -> CQRLResult<()>;
    fn get_object(&self, id: Option<String>) -> CQRLResult<Value>;
    fn store_object(&mut self, k: String, v: Value) -> CQRLResult<()>;
}

pub mod memory;

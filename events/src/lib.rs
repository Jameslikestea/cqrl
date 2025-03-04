use std::future::Future;

use errors::CQRLResult;
use persistence::SurrealObject;
use serde_json::Value;

pub trait EventEmitter<S>: Send + Sync where S: persistence::Store {
    fn run(self: &mut Self) -> impl Future<Output = CQRLResult<()>> + Send;
    fn emit(self: &mut Self, event: SurrealObject) -> CQRLResult<()>;
    fn listen(self: &mut Self, event: Value) -> CQRLResult<()>;
}

pub mod nats;
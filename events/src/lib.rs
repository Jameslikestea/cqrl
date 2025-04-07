use std::future::Future;

use errors::CQRLResult;
use futures::StreamExt;
use persistence::PersistenceObject;
use serde_json::Value;

pub trait EventEmitter<S>: Send + Sync where S: persistence::Store {
    fn run(self: &mut Self) -> impl Future<Output = CQRLResult<()>> + Send;
    fn emit(self: &mut Self, event: PersistenceObject) -> CQRLResult<()>;
    fn listen(self: &mut Self, event: Value) -> impl StreamExt<Item = PersistenceObject> + Send;
}

pub mod nats;
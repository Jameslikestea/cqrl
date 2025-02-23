use errors::CQRLResult;
use serde_json::Value;

pub trait EventEmitter: Send + Sync {
    fn emit(&self, event: Value) -> CQRLResult<()>;
}

pub trait EventListener: Send + Sync {
    fn listen(&self, event: Value) -> CQRLResult<()>;
}

pub mod nats;
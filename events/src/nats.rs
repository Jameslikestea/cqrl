use cloudevents::EventBuilder;
use errors::CQRLResult;
use persistence::SurrealObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures::StreamExt;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEventEmitter<S> where S: persistence::Store + Clone {
    store: S,
}

impl<S> NatsEventEmitter<S> where S: persistence::Store + Clone {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> crate::EventEmitter<S> for NatsEventEmitter<S> where S: persistence::Store + Clone {
    async fn run(self: &mut Self) -> CQRLResult<()> {
        let mut store = self.store.clone();
        let stream = store.watch_operation();
        let mut stream_reader = Box::pin(stream);
        while let Some(event) = stream_reader.next().await {
            let mut emitter = self.clone();
            emitter.emit(event).unwrap();
        }

        Ok(())
    }

    fn emit(self: &mut Self, event: SurrealObject) -> CQRLResult<()> {
        let source = event.metadata.get("type").unwrap_or(&Value::String(String::from("unknown"))).as_str().unwrap().to_string();
        let id = event.id.key().to_string();

        let ulid = ulid::Ulid::from_string(id.as_str()).unwrap();
        let timestamp = ulid.timestamp_ms();

        let ts = chrono::DateTime::from_timestamp_millis(timestamp.try_into().unwrap()).unwrap();

        let event = cloudevents::EventBuilderV10::new().id(event.id.key().to_string()).ty("cqrl.command").source(source.clone()).data(source.clone(), event.data).time(ts).build();
        println!("{:#?}", event);

        Ok(())
    }

    fn listen(self: &mut Self, event: Value) -> CQRLResult<()> {
        Ok(())
    }
}

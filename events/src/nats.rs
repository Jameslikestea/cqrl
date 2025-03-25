use std::sync::Arc;

use cloudevents::EventBuilder;
use errors::CQRLResult;
use persistence::PersistenceObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use futures::StreamExt;
use tokio;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEventEmitter<S> where S: persistence::Store + Clone {
    store: S,
    #[serde(skip)]
    client: Option<Arc<async_nats::Client>>,
}

impl<S> NatsEventEmitter<S> where S: persistence::Store + Clone {
    pub fn new(store: S, client: async_nats::Client) -> Self {
        Self { 
            store, 
            client: Some(Arc::new(client)) 
        }
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

    fn emit(self: &mut Self, event: PersistenceObject) -> CQRLResult<()> {
        let source = event.metadata.get("type").unwrap_or(&Value::String(String::from("unknown"))).as_str().unwrap().to_string();
        let id = event.id.key().to_string();

        println!("emitting event: {:?}", id);

        let ulid = ulid::Ulid::from_string(id.as_str()).unwrap();
        let timestamp = ulid.timestamp_ms();

        let ts = chrono::DateTime::from_timestamp_millis(timestamp.try_into().unwrap()).unwrap();

        let event = cloudevents::EventBuilderV10::new()
            .id(event.id.key().to_string())
            .ty("cqrl.command")
            .source(source.clone())
            .data(source.clone(), event.data)
            .time(ts)
            .build();

        // Clone the Arc<Client> before moving it into the spawned task
        if let Some(client) = &self.client {
            let client = client.clone();
            tokio::spawn(async move {
                client.publish(
                    format!("cqrl.command.{}", source),
                    serde_json::to_string(&event.unwrap()).unwrap().into()
                ).await
            });
        }

        Ok(())
    }

    fn listen(self: &mut Self, _event: Value) -> CQRLResult<()> {
        Ok(())
    }
}

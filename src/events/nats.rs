use std::sync::Arc;

use crate::persistence::{PersistenceObject, Store};
use cloudevents::{event::ExtensionValue, AttributesReader, Event, EventBuilder};
use errors::CQRLResult;
use futures::{channel::mpsc, SinkExt, StreamExt};
use parser::API;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEventEmitter<S>
where
    S: Store + Clone,
{
    store: S,
    #[serde(skip)]
    client: Option<Arc<async_nats::Client>>,
    #[serde(skip)]
    api: Option<Arc<API>>,
}

impl<S> NatsEventEmitter<S>
where
    S: Store + Clone,
{
    pub fn new(store: S, client: async_nats::Client, api: Arc<API>) -> Self {
        Self {
            store,
            client: Some(Arc::new(client)),
            api: Some(api),
        }
    }
}

impl<S> super::EventEmitter<S> for NatsEventEmitter<S>
where
    S: Store + Clone,
{
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
        let source = event
            .metadata
            .get("type")
            .unwrap_or(&Value::String(String::from("unknown")))
            .as_str()
            .unwrap()
            .to_string();
        let id = event.id.clone();

        println!("emitting event: {:?}", id);

        let ulid = ulid::Ulid::from_string(id.as_str()).unwrap();
        let timestamp = ulid.timestamp_ms();

        let ts = chrono::DateTime::from_timestamp_millis(timestamp.try_into().unwrap()).unwrap();

        let authtype = ExtensionValue::String(match event.metadata.get("authcontext") {
            Some(authcontext) => match authcontext.get("authtype") {
                Some(authtype) => authtype.as_str().unwrap_or("unauthenticated").to_string(),
                None => "unauthenticated".to_string(),
            },
            None => "unauthenticated".to_string(),
        });

        let event = cloudevents::EventBuilderV10::new()
            .id(event.id.clone())
            .ty("cqrl.command")
            .source(format!(
                "urn:cqrl:operation:{}:{}",
                source.clone(),
                id.clone()
            ))
            .data("application/json", event.data)
            .extension("authtype", authtype)
            .time(ts)
            .build();

        // Clone the Arc<Client> before moving it into the spawned task
        if let Some(client) = &self.client {
            let client = client.clone();
            tokio::spawn(async move {
                client
                    .publish(
                        format!("cqrl.command.{}", source),
                        serde_json::to_string(&event.unwrap()).unwrap().into(),
                    )
                    .await
            });
        }

        Ok(())
    }

    fn listen(self: &mut Self, _event: Value) -> impl StreamExt<Item = Event> + Send {
        let (mut tx, rx) = mpsc::unbounded();

        let client = self.client.clone().unwrap();
        let api = self.api.clone().unwrap();
        tokio::spawn(async move {
            let local_api = api.clone();
            let mut subscription = client.subscribe("cqrl.update.*").await.unwrap();
            while let Some(message) = subscription.next().await {
                let event = match serde_json::from_slice::<cloudevents::Event>(&message.payload) {
                    Ok(event) => {
                        // Validate the event
                        if ulid::Ulid::from_string(event.id()).is_err() {
                            warn!("Event received with invalid ID: {}, skipping. IDs should be a valid ULID", event.id());
                            continue;
                        }

                        match event.subject() {
                            Some(subject) => {
                                if ulid::Ulid::from_string(subject).is_err() {
                                    warn!("Event received with invalid subject: {}, skipping. Subjects should be a valid ULID", subject);
                                    continue;
                                }
                            }
                            None => {
                                warn!("Event received with no subject: {}, skipping. Subjects should be a valid ULID", event.id());
                                continue;
                            }
                        }

                        Arc::new(event)
                    }
                    Err(err) => {
                        warn!("Error recieving event on {}: {:?}", message.subject, err);
                        continue;
                    }
                };

                let event = match super::validate_event(event.clone(), local_api.clone()) {
                    Ok(evt) => evt,
                    Err(err) => {
                        warn!("Error validating event {}, skipping: {:?}", event.id(), err);
                        continue;
                    }
                };

                match tx.send(event.clone()).await {
                    Ok(_) => {
                        info!("Recieved event {} on {}", event.id(), message.subject);
                    }
                    Err(err) => {
                        warn!("Error recieving event on {}: {:?}", message.subject, err);
                    }
                }
            }
        });

        rx
    }
}

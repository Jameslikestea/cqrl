use cloudevents::{binding::nats::NatsCloudEvent, EventBuilder};

pub struct NatsEventEmitter {
    client: NatsCloudEvent,
}

impl NatsEventEmitter {
    pub fn new(client: NatsCloudEvent) -> Self {
        Self { client }
    }
}

impl EventEmitter for NatsEventEmitter {
    fn emit(&self, event: SurrealObject) -> CQRLResult<()> {
        let event = cloudevents::EventBuilderV10::new().id(event);

        Ok(())
    }
}

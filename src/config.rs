use serde::{Deserialize, Serialize};
use twelf::config;

#[derive(Debug, Serialize)]
#[config]
pub(crate) struct ServeConfig {
    port: u16,
    host: String,

    nats: NatsConfig,
    database: DatabaseConfig,
    telemetry: TelemetryConfig,
}

#[derive(Serialize, Debug)]
#[config]
pub(crate) struct NatsConfig {
    prefix: String,
    address: String,
}

#[derive(Serialize, Debug)]
#[config]
pub(crate) struct DatabaseConfig {
    mode: String,
    connection_string: String,
}

#[derive(Serialize, Debug)]
#[config]
pub(crate) struct TelemetryConfig {
    enabled: bool,
    endpoint: String,
}

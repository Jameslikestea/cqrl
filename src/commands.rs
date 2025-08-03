use std::sync::Arc;
use std::{error::Error, fs, time::Duration};

use crate::events::EventEmitter;
use crate::persistence::{Permission, PermissionStore, Store};
use crate::server_v2;
use crate::{
    commands_generate::GenerateCommand, events::nats::NatsEventEmitter,
    persistence::mongo::MongoStore,
};
use clap::Subcommand;
use cloudevents::{AttributesReader, Data};
use errors::CQRLError;
use futures::StreamExt;
use mongodb::options::ClientOptions;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use parser::{parse_hcl::parse_hcl, API};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, instrument, warn};

#[derive(Debug, Clone, Subcommand, Deserialize, Serialize)]
pub(crate) enum Commands {
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Serve {
        #[arg(
            required(true),
            value_name("SERVICE_FILE"),
            help = "The service file to serve"
        )]
        input: String,
        #[arg(long, short, default_value_t = String::from("mongodb"))]
        database_mode: String,
        #[arg(long, short, default_value_t = String::from("mongodb://cqrl:cqrl@localhost:27017/cqrl"))]
        mongodb_address: String,
        #[arg(long, short, default_value_t = String::from("nats://localhost:4222"))]
        nats_address: String,
        #[arg(long = "jwks-endpoint")]
        jwks_url: Option<String>,
    },
}

impl Default for Commands {
    fn default() -> Self {
        Self::Serve {
            input: String::from("./service.hcl"),
            database_mode: String::from("mongodb"),
            mongodb_address: String::from("mongodb://cqrl:cqrl@localhost:27017/cqrl"),
            nats_address: String::from("nats://localhost:4222"),
            jwks_url: None,
        }
    }
}

fn setup_metrics_exporter() {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4318/v1/metrics")
        .build()
        .unwrap();
    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build();

    opentelemetry::global::set_meter_provider(provider);

    let meter = global::meter("cqrl-server");
    let up = meter.u64_gauge("up").build();
    up.record(1, &[KeyValue::new("app", "cqrl-server")]);
}

impl Commands {
    #[instrument(skip(self))]
    pub(crate) async fn run(self: Self) -> Result<(), Box<dyn Error>> {
        match self {
            Commands::Generate { command } => command.run().await,
            Commands::Serve {
                input,
                database_mode,
                mongodb_address,
                nats_address,
                jwks_url,
            } => {
                {
                    info!("Serving CQRL for {}", input);
                }
                info!("Setting up metrics exporter");
                setup_metrics_exporter();
                info!("Metrics exporter setup");
                let mut content: String = String::new();

                match fs::read_to_string(input.clone()) {
                    Ok(file) => content = file,
                    Err(err) => {
                        error!("Cannot read input file `{}`: {}", input, err);
                    }
                };

                let mut api: Arc<API> = Arc::new(API::new());

                match parse_hcl(&content) {
                    Ok(parsed_api) => {
                        api = Arc::new(parsed_api);
                    }
                    Err(err) => {
                        error!("Cannot parse input file: `{}`: {}", input, err);
                    }
                };

                let nats_client = async_nats::connect(nats_address).await.unwrap();

                match database_mode.as_str() {
                    "mongodb" => {
                        let mut options =
                            ClientOptions::parse(mongodb_address.clone()).await.unwrap();
                        options.connect_timeout = Some(Duration::from_secs(5));
                        options.server_selection_timeout = Some(Duration::from_secs(5));
                        let client = mongodb::Client::with_options(options).unwrap();
                        let mut store = MongoStore::new(client.clone(), api.clone());
                        store.init().await.unwrap();

                        let send_client = client.clone();
                        let recv_client = client.clone();

                        let send_nats = nats_client.clone();
                        let recv_nats = nats_client.clone();

                        let send_api = api.clone();
                        let recv_api = api.clone();

                        let _handle = tokio::spawn(async move {
                            let mut local_sender = NatsEventEmitter::new(
                                MongoStore::new(send_client, send_api.clone()),
                                send_nats,
                                send_api,
                            );
                            local_sender.run().await.unwrap();
                        });
                        let _handle_listen = tokio::spawn(async move {
                            let mut local_store = MongoStore::new(recv_client, recv_api.clone());
                            let mut local_sender =
                                NatsEventEmitter::new(local_store.clone(), recv_nats, recv_api);
                            let mut stream = local_sender.listen(Value::Null);
                            while let Some(event) = stream.next().await {
                                debug!("Recieved event: {}", event.id());

                                match local_store
                                    .store_object(
                                        event.subject().unwrap().to_string(),
                                        event.clone(),
                                    )
                                    .await
                                {
                                    Ok(_) => (),
                                    Err(err) => {
                                        warn!("Error storing event: {:?}", err);
                                    }
                                };
                            }
                        });

                        let recv_client = client.clone();
                        let recv_nats = nats_client.clone();
                        let recv_api = api.clone();

                        tokio::spawn(async move {
                            let local_store = MongoStore::new(recv_client, recv_api.clone());
                            let mut local_sender =
                                NatsEventEmitter::new(local_store.clone(), recv_nats, recv_api);

                            let mut stream = local_sender.listen_permission(Value::Null);

                            while let Some(evt) = stream.next().await {
                                debug!("Recieved permission event: {}: {:?}", evt.id(), evt.data());
                                let data = match evt.data().unwrap() {
                                    Data::Json(json) => json,
                                    _ => continue,
                                };
                                match data.get("type").unwrap().as_str().unwrap() {
                                    "permit" => {
                                        let _ = local_store
                                            .clone()
                                            .grant(
                                                evt.subject().unwrap().to_string(),
                                                evt.extension("authid").unwrap().to_string(),
                                                match data.get("level").unwrap().as_str().unwrap() {
                                                    "write" => Permission::Write,
                                                    _ => Permission::Read,
                                                },
                                            )
                                            .await;
                                    }
                                    "deny" => {
                                        let _ = local_store
                                            .clone()
                                            .revoke(
                                                evt.subject().unwrap().to_string(),
                                                evt.extension("authid").unwrap().to_string(),
                                                match data.get("level").unwrap().as_str().unwrap() {
                                                    "write" => Permission::Write,
                                                    _ => Permission::Read,
                                                },
                                            )
                                            .await;
                                    }
                                    _ => {}
                                }
                            }
                        });

                        server_v2::run(api.clone(), Arc::new(client.clone()), jwks_url).await;
                    }
                    mode => {
                        error!("Unsupported database type: {}", mode);
                    }
                };

                Ok(())
            }
        }
    }
}

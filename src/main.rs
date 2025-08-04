use std::error::Error;

use clap::Parser;
use clap_serde_derive::ClapSerde;
use opentelemetry_appender_tracing::layer;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{logs::SdkLoggerProvider, Resource};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

mod chains;
mod commands;
mod commands_generate;
mod events;
mod openapigenerator;
mod persistence;
mod server_v2;

#[derive(Parser, Debug, Clone, clap_serde_derive::ClapSerde, Deserialize, Serialize)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
    #[arg(long)]
    otlp_endpoint: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let filter_fmt = EnvFilter::new("info").add_directive("cqrl=debug".parse().unwrap());
    let formatter = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .json()
        .with_filter(filter_fmt);
    let layers = tracing_subscriber::registry().with(formatter);

    if args.otlp_endpoint.is_some() {
        let exporter = opentelemetry_otlp::LogExporter::builder()
            .with_tonic()
            .with_endpoint(format!("{}/v1/logs", args.otlp_endpoint.clone().unwrap()))
            .build()
            .unwrap();
        let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
            .with_resource(Resource::builder().with_service_name("cqrl-server").build())
            .with_batch_exporter(exporter)
            .build();

        let filter_otel = EnvFilter::new("debug")
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("tonic=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap())
            .add_directive("actix_web=off".parse().unwrap())
            .add_directive("tower=off".parse().unwrap())
            .add_directive("async_nats=off".parse().unwrap());

        let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);
        layers.with(otel_layer).init();
    } else {
        layers.init();
    }

    let _ = ctrlc::set_handler(move || {
        println!("Goodbye!");
        std::process::exit(0);
    });

    let command = args.command.clone();
    let otlp_endpoint = args.otlp_endpoint.clone();
    command.run(otlp_endpoint.clone()).await
}

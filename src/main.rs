use std::error::Error;

use clap::Parser;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod chains;
mod commands;
mod commands_generate;
mod events;
mod openapigenerator;
mod persistence;
mod server_v2;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let formatter = tracing_subscriber::fmt::layer().with_level(true).with_target(true).with_thread_ids(false).with_thread_names(false).json();
    tracing_subscriber::registry().with(tracing::level_filters::LevelFilter::from_level(Level::INFO)).with(formatter).init();

    let _ = ctrlc::set_handler(move || {
        println!("Goodbye!");
        std::process::exit(0);
    });

    args.command.run().await
}

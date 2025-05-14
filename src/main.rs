use std::error::Error;

use clap::Parser;

mod chains;
mod commands;
mod commands_generate;
mod events;
mod openapigenerator;
mod persistence;
mod server;
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

    let _ = ctrlc::set_handler(move || {
        println!("Goodbye!");
        std::process::exit(0);
    });

    args.command.run().await
}

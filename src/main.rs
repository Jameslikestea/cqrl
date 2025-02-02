use std::error::Error;

use clap::Parser;

mod commands;
mod commands_generate;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    args.command.run().await
}

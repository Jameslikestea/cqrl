use std::error::Error;

use crate::commands_generate::GenerateCommand;
use clap::Subcommand;
use server::Server;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Commands {
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Serve,
}

impl Commands {
    pub(crate) async fn run(self: Self) -> Result<(), Box<dyn Error>> {
        match self {
            Commands::Generate { command } => command.run().await,
            Commands::Serve => {
                let server = Server::new();
                server.serve().await
            }
        }
    }
}

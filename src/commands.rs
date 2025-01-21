use crate::commands_generate::GenerateCommand;
use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Commands {
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Serve,
}

impl Commands {
    pub(crate) fn run(self: Self) {
        match self {
            Commands::Generate { command } => command.run(),
            Commands::Serve => {}
        }
    }
}

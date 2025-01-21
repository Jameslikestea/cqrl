use clap::Parser;

mod commands;
mod commands_generate;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about=None)]
struct Args {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() {
    let args = Args::parse();

    args.command.run();
}

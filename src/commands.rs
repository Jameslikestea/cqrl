use std::{error::Error, fs};

use crate::commands_generate::GenerateCommand;
use clap::Subcommand;
use parser::{CQRLParser, API};
use server::Server;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Commands {
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Serve {
        #[arg(long, short, default_value_t = String::from("./service.cqrl"))]
        input: String,
    },
}

impl Commands {
    pub(crate) async fn run(self: Self) -> Result<(), Box<dyn Error>> {
        match self {
            Commands::Generate { command } => command.run().await,
            Commands::Serve { input } => {
                {
                    println!("Serving CQRL for {}", input);
                }
                let mut content: String = String::new();

                match fs::read_to_string(input.clone()) {
                    Ok(file) => content = file,
                    Err(err) => {
                        println!("Cannot read input file `{}`: {}", input, err);
                    }
                };

                let mut api: API = API::new();

                match CQRLParser::parse_string(&content) {
                    Ok(parsed_api) => {
                        api = parsed_api;
                    }
                    Err(err) => {
                        println!("Cannot parse input file: `{}`: {}", input, err);
                    }
                };



                let db = surrealdb::Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();
                db.signin(Root {
                    username: "root",
                    password: "root",
                }).await?;
                db.use_ns("test").use_db("test").await?;

                let mut store = persistence::surreal::SurrealStore::new(db);
                store.init().await?;
                let mut server = Server::new(store);

                server.with_api(api);

                server.serve().await;
                Ok(())
            }
        }
    }
}

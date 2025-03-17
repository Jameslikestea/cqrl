use std::{error::Error, fs};

use crate::commands_generate::GenerateCommand;
use clap::Subcommand;
use events::EventEmitter;
use parser::{parse_hcl::parse_hcl, API};
use server::Server;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum Commands {
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Serve {
        #[arg(long, short, default_value_t = String::from("./service.hcl"))]
        input: String,
        #[arg(long, short, default_value_t = String::from("mongodb"))]
        database_mode: String,
        #[arg(long, short, default_value_t = String::from("mongodb://cqrl:cqrl@localhost:27017/cqrl"))]
        mongodb_address: String,
        #[arg(long, short, default_value_t = String::from("ws://localhost:8000"))]
        surreal_address: String,
        #[arg(long, short, default_value_t = String::from("nats://localhost:4222"))]
        nats_address: String,
    },
}

impl Commands {
    pub(crate) async fn run(self: Self) -> Result<(), Box<dyn Error>> {
        match self {
            Commands::Generate { command } => command.run().await,
            Commands::Serve { input, database_mode, mongodb_address, surreal_address, nats_address } => {
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

                match parse_hcl(&content) {
                    Ok(parsed_api) => {
                        api = parsed_api;
                    }
                    Err(err) => {
                        println!("Cannot parse input file: `{}`: {}", input, err);
                    }
                };

                let nats_client = async_nats::connect(nats_address).await.unwrap();
                
                match database_mode.as_str() {
                    "surreal" => {
                        let db = surrealdb::Surreal::new::<Ws>(surreal_address.clone()).await.unwrap();
                        db.signin(Root{
                            username: "root",
                            password: "root",
                        }).await?;
                        db.use_ns("test").use_db("test").await?;
                        let mut server = Server::new(persistence::surreal::SurrealStore::new(db.clone()));
                        server.with_api(api);

                        let handle = tokio::spawn(async move {
                            let mut sender = events::nats::NatsEventEmitter::new(persistence::surreal::SurrealStore::new(db.clone()), nats_client.clone());
                            sender.run().await.unwrap();
                        });

                        server.serve().await;
                        handle.await?;
                    },
                    "mongodb" => {
                        let client = mongodb::Client::with_uri_str(mongodb_address.clone()).await.unwrap();
                        let mut server = Server::new(persistence::mongo::MongoStore::new(client.clone()));
                        server.with_api(api);

                        let handle = tokio::spawn(async move {
                            let mut sender = events::nats::NatsEventEmitter::new(persistence::mongo::MongoStore::new(client.clone()), nats_client.clone());
                            sender.run().await.unwrap();
                        });
                        server.serve().await;
                        handle.await?;
                    },
                    mode => {
                        println!("Unsupported database type: {}", mode);
                    }
                };

                Ok(())
            }
        }
    }
}

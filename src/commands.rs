use std::{error::Error, fs, time::Duration};

use crate::{commands_generate::GenerateCommand, events::nats::NatsEventEmitter, persistence::mongo::MongoStore};
use clap::Subcommand;
use cloudevents::AttributesReader;
use crate::events::EventEmitter;
use mongodb::options::ClientOptions;
use parser::{parse_hcl::parse_hcl, API};
use serde_json::Value;
use futures::StreamExt;
use crate::server::Server;
use crate::persistence::Store;

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
                    "mongodb" => {
                        let mut options = ClientOptions::parse(mongodb_address.clone()).await.unwrap();
                        options.connect_timeout = Some(Duration::from_secs(5));
                        options.server_selection_timeout = Some(Duration::from_secs(5));
                        let client = mongodb::Client::with_options(options).unwrap();
                        let mut store = MongoStore::new(client.clone());
                        store.init().await.unwrap();
                        let mut server = Server::new(store);
                        server.with_api(api);

                        let send_client = client.clone();
                        let recv_client = client.clone();

                        let send_nats = nats_client.clone();
                        let recv_nats = nats_client.clone();

                        let handle = tokio::spawn(async move {
                            let mut local_sender = crate::events::nats::NatsEventEmitter::new(MongoStore::new(send_client), send_nats);
                            local_sender.run().await.unwrap();
                        });
                        let handle_listen = tokio::spawn(async move {
                            let mut local_store = MongoStore::new(recv_client);
                            let mut local_sender = NatsEventEmitter::new(local_store.clone(), recv_nats);
                            let mut stream = local_sender.listen(Value::Null);
                            while let Some(event) = stream.next().await {
                                println!("Recieved event: {:?}", event);
                                match local_store.store_object(event.id().to_string(), event.clone()).await {
                                    Ok(_) => {
                                        println!("Stored event: {:?}", event.id());
                                    },
                                    Err(err) => {
                                        println!("Error storing event: {:?}", err);
                                    }
                                };
                            }
                        });
                        server.serve().await;
                        handle.await?;
                        handle_listen.await?;
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

use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use parser::API;

use crate::chains;

pub (crate) async fn run(api: Arc<API>, store: Arc<mongodb::Client>) {
    let query_chain = chains::ProcessingChain::new(vec![
        Arc::new(chains::url::URLChain),
        Arc::new(chains::methods::QueryMethod::new(api.clone())),
        Arc::new(chains::persistence::MongoQueryChain::new(api.clone(), store.clone())),
    ]);

    let command_chain = chains::ProcessingChain::new(vec![
        Arc::new(chains::url::URLChain),
        Arc::new(chains::methods::CommandMethod::new(api.clone()))
    ]);

    let server = HttpServer::new(move|| {
        App::new()
            .route("/query/{method}", web::get().to(query_chain.clone()))
            .route("/command/{method}", web::post().to(command_chain.clone()))
    });

    let _ = server.bind("127.0.0.1:8000").unwrap().run().await;
}

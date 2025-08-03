use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use parser::API;
use tracing::instrument;

use crate::chains;

#[instrument(skip(api, store))]
pub(crate) async fn run(api: Arc<API>, store: Arc<mongodb::Client>, jwks_url: Option<String>) {
    let query_chain = chains::ProcessingChain::new(vec![
        Arc::new(chains::request::HeaderChain),
        Arc::new(chains::auth::AuthChain::new(jwks_url.clone())),
        Arc::new(chains::ratelimit::RateLimitChain::new(store.clone())),
        Arc::new(chains::log::LogChain),
        Arc::new(chains::url::URLChain),
        Arc::new(chains::methods::QueryMethod::new(api.clone())),
        Arc::new(chains::persistence::MongoQueryChain::new(
            api.clone(),
            store.clone(),
        )),
    ]);

    let command_chain = chains::ProcessingChain::new(vec![
        Arc::new(chains::request::HeaderChain),
        Arc::new(chains::auth::AuthChain::new(jwks_url.clone())),
        Arc::new(chains::ratelimit::RateLimitChain::new(store.clone())),
        Arc::new(chains::log::LogChain),
        Arc::new(chains::url::URLChain),
        Arc::new(chains::methods::CommandMethod::new(api.clone())),
        Arc::new(chains::persistence::MongoCommandChain::new(
            api.clone(),
            store.clone(),
        )),
    ]);

    let head_chain = chains::ProcessingChain::new(vec![
        Arc::new(chains::request::HeaderChain),
        Arc::new(chains::auth::AuthChain::new(jwks_url.clone())),
        Arc::new(chains::log::LogChain),
        Arc::new(chains::url::URLChain),
        Arc::new(chains::methods::QueryMethod::new(api.clone())),
        Arc::new(chains::persistence::MongoQueryChain::new(
            api.clone(),
            store.clone(),
        )),
    ]);

    let server = HttpServer::new(move || {
        App::new()
            .route("/query/{method}", web::get().to(query_chain.clone()))
            .route("/query/{method}", web::head().to(head_chain.clone()))
            .route("/command/{method}", web::post().to(command_chain.clone()))
    });

    let _ = server.bind("127.0.0.1:8912").unwrap().run().await;
}

use axum::{
    routing::{get, post},
    Router,
};

use parser::API;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server {
    port: u16,
    api: Arc<API>,
    state: ServerState,
}

impl Server {
    pub fn new() -> Self {
        Server {
            port: 8912,
            api: Arc::new(API::new()),
            state: ServerState::new(Arc::new(API::new())),
        }
    }

    pub fn with_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn with_api(&mut self, api: API) {
        self.api = Arc::new(api);
        self.state.api = self.api.clone();
    }

    pub async fn serve(&self) -> () {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await.unwrap();
        let api = self.api.clone();

        let mut router = Router::new();

        router = router.route("/", get(handlers::root));

        for command in api.commands.iter() {
            router = router.route(
                format!("/command/{}", command.name).as_str(),
                post(handlers::command),
            );
            println!("Discovered Command: {}", command.name);
        }
        for query in api.queries.iter() {
            router = router.route(
                format!("/query/{}", query.name).as_str(),
                get(handlers::query),
            );
            println!("Discoviered Query: {}", query.name);
        }

        let state = self.state.clone();

        axum::serve(listener, router.with_state(state))
            .await
            .unwrap()
    }
}

mod handlers {
    use axum::{extract::State, response::IntoResponse, Json};
    use hyper::StatusCode;
    use serde_json::json;

    use crate::ServerState;

    pub async fn root(_state: State<ServerState>) -> impl IntoResponse {
        (StatusCode::OK, Json("hello, world!"))
    }

    pub async fn command(State(_state): State<ServerState>) -> impl IntoResponse {
        (StatusCode::ACCEPTED, [("x-command-id", "1234")])
    }

    pub async fn query(State(_state): State<ServerState>) -> impl IntoResponse {
        (
            StatusCode::OK,
            [("x-pagination-token", "1234")],
            Json(json!({
                "hello": "world"
            })),
        )
    }
}

#[derive(Debug, Clone)]
struct ServerState {
    api: Arc<API>,
}

impl ServerState {
    pub fn new(api: Arc<API>) -> Self {
        ServerState { api }
    }
}

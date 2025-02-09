use axum::{
    routing::{get, post},
    Router,
};

use parser::API;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Server<S> where S: persistence::Store + Clone + Send + Sync + 'static {
    port: u16,
    state: ServerState<S>,
}

impl<S> Server<S> where S: persistence::Store + Clone + Send + Sync + 'static {
    pub fn new(store: S) -> Self {
        Server {
            port: 8912,
            state: ServerState::new(Arc::new(API::new()), store),
        }
    }

    pub fn with_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn with_api(&mut self, api: API) {
        self.state.api = Arc::new(api);
    }

    pub async fn serve(&self) -> () {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        let listener = TcpListener::bind(addr).await.unwrap();

        let mut router = Router::new();

        router = router.route("/", get(handlers::root))
            .route("/command/{command}", post(handlers::command))
            .route("/query/{query}", get(handlers::query));

        let state = self.state.clone();

        axum::serve(listener, router.with_state(state))
            .await
            .unwrap()
    }
}

mod handlers {
    use axum::{body::Body, extract::{Path, State}, response::{IntoResponse, Response}, Json};
    use hyper::StatusCode;
    use serde_json::json;

    use crate::ServerState;

    pub async fn root<S>(_state: State<ServerState<S>>) -> impl IntoResponse where S: persistence::Store + Clone + Send + Sync + 'static {
        (StatusCode::OK, Json("hello, world!"))
    }
    pub async fn command<S>(Path(_command): Path<String>, State(mut _state): State<ServerState<S>>) -> impl IntoResponse 
    where 
        S: persistence::Store + Clone + Send + Sync + 'static,
    {
        if !_state.api.commands.iter().any(|c| c.name == _command) {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
        }

        match _state.store.store_operation("some_command".to_string(), json!("Some Command"), "test_object".to_string()).await {
            Ok(_) => {
                Response::builder()
                    .status(StatusCode::ACCEPTED)
                    .header("x-command-id", "1234")
                    .body(Body::empty())
                    .unwrap()
            },
            Err(e) => {
                println!("Error storing command: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap()
            }
        }
        
    }
    pub async fn query<S>(Path(_query): Path<String>, State(_state): State<ServerState<S>>) -> impl IntoResponse 
    where 
        S: persistence::Store + Clone + Send + Sync + 'static,
    {
        if !_state.api.queries.iter().any(|q| q.name == _query) {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap();
        }

        let values = _state.store.get_object(None, "test_object".to_string()).await.unwrap();

        Response::builder()
            .status(StatusCode::OK)
            .header("x-pagination-token", "1234")
            .body(Body::from(Json(values).to_string()))
            .unwrap()
    }
}

#[derive(Debug, Clone)]
struct ServerState<S> where S: persistence::Store + Clone + Send + Sync + 'static {
    api: Arc<API>,
    store: S,
}

impl<S> ServerState<S> where S: persistence::Store + Clone + Send + Sync + 'static {
    pub fn new(api: Arc<API>, store: S) -> Self {
        ServerState { 
            api,
            store// You'll need to provide the persistence implementation
        }
    }
}

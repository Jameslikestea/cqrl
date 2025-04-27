use axum::{
    routing::{get, post},
    Router,
};

use parser::API;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

use crate::persistence::Store;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Server<S> where S: Store + Clone + Send + Sync + 'static {
    port: u16,
    api: Arc<API>,
    state: ServerState<S>,
}

impl<S> Server<S> where S: Store + Clone + Send + Sync + 'static {
    pub fn new(store: S) -> Self {
        Server {
            port: 8912,
            api: Arc::new(API::new()),
            state: ServerState::new(Arc::new(API::new()), store),
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

        let mut router = Router::new();

        router = router.route("/", get(handlers::root)).route("/command/{command}", post(handlers::command)).route("/query/{query}", get(handlers::query));

        let state = self.state.clone();

        axum::serve(listener, router.with_state(state))
            .await
            .unwrap()
    }
}

mod handlers {
    use axum::{body::Body, extract::{Path, State}, response::{IntoResponse, Response}, Json};
    use hyper::StatusCode;


    use crate::persistence::Store;

    use super::ServerState;

    pub async fn root<S>(_state: State<ServerState<S>>) -> impl IntoResponse where S: Store + Clone + Send + Sync + 'static {
        (StatusCode::OK, Json("hello, world!"))
    }
    pub async fn command<S>(State(mut _state): State<ServerState<S>>, Path((command_type,)): Path<(String,)>, Json(_command): Json<serde_json::Value>) -> impl IntoResponse 
    where 
        S: Store + Clone + Send + Sync + 'static,
    {
        match _state.store.store_operation(command_type.clone(), _command, command_type.clone()).await {
            Ok(object) => {
                Response::builder()
                    .status(StatusCode::ACCEPTED)
                    .header("x-command-id", object.id)
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
        S: Store + Clone + Send + Sync + 'static,
    {

        let values = _state.store.get_object(None, _query).await.unwrap();

        (
            StatusCode::OK,
            [("x-pagination-token", "1234")],
            Json(values),
        )
    }
}

#[derive(Debug, Clone)]
struct ServerState<S> where S: Store + Clone + Send + Sync + 'static {
    api: Arc<API>,
    store: S,
}

impl<S> ServerState<S> where S: Store + Clone + Send + Sync + 'static {
    pub fn new(api: Arc<API>, store: S) -> Self {
        ServerState { 
            api,
            store// You'll need to provide the persistence implementation
        }
    }
}

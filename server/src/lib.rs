use axum::{
    routing::{get, post},
    Router,
};

use parser::API;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Server<S> where S: persistence::Store + Clone + Send + Sync + 'static {
    port: u16,
    api: Arc<API>,
    state: ServerState<S>,
}

impl<S> Server<S> where S: persistence::Store + Clone + Send + Sync + 'static {
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

    pub async fn root<S>(_state: State<ServerState<S>>) -> impl IntoResponse where S: persistence::Store + Clone + Send + Sync + 'static {
        (StatusCode::OK, Json("hello, world!"))
    }
    pub async fn command<S>(State(mut _state): State<ServerState<S>>) -> impl IntoResponse where S: persistence::Store + Clone + Send + Sync + 'static {
        _state.store.store_operation("some_command".to_string(), json!("Some Command")).unwrap();
        
        (StatusCode::ACCEPTED, [("x-command-id", "1234")])
    }
    pub async fn query<S>(State(_state): State<ServerState<S>>) -> impl IntoResponse where S: persistence::Store + Clone + Send + Sync + 'static {

        let values = _state.store.get_object(None).unwrap();

        (
            StatusCode::OK,
            [("x-pagination-token", "1234")],
            Json(values),
        )
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

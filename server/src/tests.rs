use super::*;
use parser::{Command, Query};
use persistence::{memory::MemoryStore, Store};
use serde_json::json;

#[tokio::test]
async fn test_server_creation() {
    let store = MemoryStore::new();
    let server = Server::new(store);
    
    assert_eq!(server.port, 8912); // Default port
}

#[tokio::test]
async fn test_server_with_port() {
    let store = MemoryStore::new();
    let mut server = Server::new(store);
    
    server.with_port(9000);
    assert_eq!(server.port, 9000);
}

#[tokio::test]
async fn test_server_with_api() {
    let store = MemoryStore::new();
    let mut server = Server::new(store);
    
    let api = API {
        commands: vec![Command {
            name: "test".to_string(),
            modelled_by: "test_model".to_string(),
        }],
        queries: vec![Query {
            name: "test".to_string(),
            modelled_by: "test_model".to_string(),
        }],
        models: vec![],
    };
    
    server.with_api(api);
}

mod handler_tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use parser::parse_hcl::parse_hcl;

    #[tokio::test]
    async fn test_root_handler() {
        let store = MemoryStore::new();
        let state = ServerState::new(Arc::new(API::new()), store);
        
        let response = handlers::root(axum::extract::State(state)).await;
        let (status, _) = response.into_response().into_parts();
        
        assert_eq!(status.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_command_handler() {
        let store = MemoryStore::new();

        let api_hcl = r#"
        command "test" {
            modelled_by = model.test_model
        }
        model "test_model" {
            test_property = {
                type = "string"
            }
        }
        "#;

        let api = parse_hcl(api_hcl).unwrap();

        let state = ServerState::new(Arc::new(api), store);
        
        let response = handlers::command(
            axum::extract::State(state),
        ).await;
        let response = response.into_response();
        
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert_eq!(
            response.headers().get("x-command-id").unwrap(),
            "1234"
        );
    }

    #[tokio::test]
    async fn test_query_handler() {
        let mut store = MemoryStore::new();
        // Pre-populate store with test data
        store.store_object("test".to_string(), json!({"test": "value"}), "command".to_string() ).await.unwrap();

        let api_hcl = r#"
        query "test" {
            modelled_by = model.test_model
        }
        model "test_model" {
            test = {
                type = "string"
            }
        }
        "#;

        let api = parse_hcl(api_hcl).unwrap();
        
        let state = ServerState::new(Arc::new(api), store);
        let response = handlers::query(axum::extract::Path("test".to_string()), axum::extract::State(state)).await;
        let (head, _) = response.into_response().into_parts();
        
        assert_eq!(head.status, StatusCode::OK);
        assert!(head.headers.contains_key("x-pagination-token"));
    }
}

#[test]
fn test_server_state() {
    let store = MemoryStore::new();
    let api = Arc::new(API::new());
    let state = ServerState::new(api.clone(), store);
    
    assert!(state.api.commands.is_empty());
    assert!(state.api.queries.is_empty());
    assert!(state.api.models.is_empty());
}

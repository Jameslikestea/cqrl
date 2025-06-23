use cloudevents::{Data, Event};
use errors::CQRLResult;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use super::{Permission, PermissionStore, PersistenceObject, Store};

#[derive(Clone)]
pub struct MemoryStore {
    object_store: Arc<RwLock<HashMap<String, Value>>>,
    operation_store: Arc<RwLock<HashMap<String, Value>>>,
    permission_store: Arc<RwLock<HashMap<String, HashMap<String, Vec<Permission>>>>>,
}

impl MemoryStore {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            object_store: Arc::new(RwLock::new(HashMap::new())),
            operation_store: Arc::new(RwLock::new(HashMap::new())),
            permission_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Store for MemoryStore {
    async fn get_object(&self, k: Option<String>, _: String) -> CQRLResult<Value> {
        let store = self
            .object_store
            .read()
            .map_err(|_| errors::CQRLError::Generic)?;
        match k {
            Some(key) => match store.get(&key) {
                Some(v) => Ok(v.clone()),
                None => Err(errors::CQRLError::Generic),
            },
            None => {
                println!(
                    "Getting all objects: {:?}",
                    store.iter().collect::<Vec<_>>()
                );
                Ok(Value::Array(store.values().cloned().collect()))
            }
        }
    }

    async fn store_object(&mut self, k: String, v: Event) -> CQRLResult<()> {
        println!("Storing object: {:?}", k);
        let mut store = self
            .object_store
            .write()
            .map_err(|_| errors::CQRLError::StoreError {
                error: "Cannot store object in store".to_string(),
            })?;
        store.insert(
            k,
            match v.data() {
                Some(data) => match data {
                    Data::Json(json) => json.clone(),
                    Data::String(string) => serde_json::from_str(&string).unwrap(),
                    Data::Binary(binary) => serde_json::from_slice(&binary).unwrap(),
                },
                None => serde_json::Value::from_str("").unwrap(),
            },
        );
        Ok(())
    }

    async fn store_operation(
        &mut self,
        k: String,
        v: Value,
        _: String,
    ) -> CQRLResult<super::PersistenceObject> {
        let mut store =
            self.operation_store
                .write()
                .map_err(|_| errors::CQRLError::StoreError {
                    error: "Cannot store operation in store".to_string(),
                })?;
        store.insert(k.clone(), v.clone());
        Ok(super::PersistenceObject {
            id: k.clone(),
            data: v.clone(),
            metadata: Value::Null,
        })
    }

    fn watch_operation(&mut self) -> impl Stream<Item = PersistenceObject> + Send {
        futures::stream::empty::<PersistenceObject>()
    }
}

impl PermissionStore for MemoryStore {
    async fn _permit(&self, id: String, user: String, permission: Permission) -> CQRLResult<bool> {
        let store = self
            .permission_store
            .read()
            .map_err(|_| errors::CQRLError::Generic)?;
        match store.get(&id) {
            Some(permissions) => match permissions.get(&user) {
                Some(p) => Ok(p.contains(&permission)),
                None => Ok(false),
            },
            None => Ok(false),
        }
    }

    async fn _grant(&mut self, id: String, user: String, permission: Permission) -> CQRLResult<()> {
        let mut store =
            self.permission_store
                .write()
                .map_err(|_| errors::CQRLError::StoreError {
                    error: "Cannot store permission in store".to_string(),
                })?;
        let permissions = store.entry(id).or_insert(HashMap::new());
        let user_permissions = permissions.entry(user).or_insert(Vec::new());
        if !user_permissions.contains(&permission) {
            user_permissions.push(permission);
        }
        Ok(())
    }

    async fn _revoke(
        &mut self,
        id: String,
        user: String,
        permission: Permission,
    ) -> CQRLResult<()> {
        let mut store =
            self.permission_store
                .write()
                .map_err(|_| errors::CQRLError::StoreError {
                    error: "Cannot store permission in store".to_string(),
                })?;
        if let Some(permissions) = store.get_mut(&id) {
            if let Some(user_permissions) = permissions.get_mut(&user) {
                if let Some(pos) = user_permissions.iter().position(|p| p == &permission) {
                    user_permissions.remove(pos);
                }
                if user_permissions.is_empty() {
                    permissions.remove(&user);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryStore;
    use super::Store;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use serde_json::json;

    #[tokio::test]
    async fn test_store_object() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_object(
                    "value".to_string(),
                    EventBuilderV10::new()
                        .id("ABCDEF")
                        .ty("test")
                        .data(
                            "application/json",
                            json!({
                                "test": "value"
                            })
                        )
                        .source("ABCDEF")
                        .build()
                        .unwrap()
                )
                .await
                .is_ok(),
            true
        );

        assert_eq!(store.object_store.read().unwrap().len(), 1);
        assert_eq!(
            store.object_store.read().unwrap().get(&"value".to_string()),
            Some(&json!({
                "test": "value"
            }))
        );
    }

    #[tokio::test]
    async fn test_store_operation() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_operation(
                    "value".to_string(),
                    json!({
                        "test": "value"
                    }),
                    "operation".to_string()
                )
                .await
                .is_ok(),
            true
        );

        assert_eq!(store.operation_store.read().unwrap().len(), 1);
        assert_eq!(
            store
                .operation_store
                .read()
                .unwrap()
                .get(&"value".to_string()),
            Some(&json!({
                "test": "value"
            }))
        );
    }

    #[tokio::test]
    async fn end_to_end() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_object(
                    "value".to_string(),
                    EventBuilderV10::new()
                        .id("ABCDEF")
                        .ty("test")
                        .data(
                            "application/json",
                            json!({
                                "test": "value"
                            })
                        )
                        .source("ABCDEF")
                        .build()
                        .unwrap(),
                )
                .await
                .is_ok(),
            true
        );

        assert_eq!(store.object_store.read().unwrap().len(), 1);
        let mut result = store
            .get_object(Some("value".to_string()), "object".to_string())
            .await;
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(
            value,
            json!({
                "test": "value"
            })
        );
        result = store
            .get_object(Some("invalid".to_string()), "object".to_string())
            .await;
        assert!(result.is_err());
    }
}

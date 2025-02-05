use errors::CQRLResult;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde_json::Value;

use crate::{Permission, PermissionStore, Store};

#[derive(Clone)]
pub struct MemoryStore {
    object_store: Arc<RwLock<HashMap<String, Value>>>,
    operation_store: Arc<RwLock<HashMap<String, Value>>>,
    permission_store: Arc<RwLock<HashMap<String, HashMap<String, Vec<Permission>>>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            object_store: Arc::new(RwLock::new(HashMap::new())),
            operation_store: Arc::new(RwLock::new(HashMap::new())),
            permission_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Store for MemoryStore {
    fn get_object(&self, k: Option<String>) -> CQRLResult<Value> {
        let store = self.object_store.read().map_err(|_| errors::CQRLError::Generic)?;
        match k {
            Some(key) => match store.get(&key) {
                Some(v) => Ok(v.clone()),
                None => Err(errors::CQRLError::Generic),
            }
            None => {
                println!("Getting all objects: {:?}", store.iter().collect::<Vec<_>>());
                Ok(Value::Array(store.values().cloned().collect()))
            },
        }
    }

    fn store_object(&mut self, k: String, v: Value) -> CQRLResult<()> {
        println!("Storing object: {:?}", k);
        let mut store = self.object_store.write().map_err(|_| errors::CQRLError::StoreError)?;
        store.insert(k, v);
        Ok(())
    }

    fn store_operation(&mut self, k: String, v: Value) -> CQRLResult<()> {
        let mut store = self.operation_store.write().map_err(|_| errors::CQRLError::StoreError)?;
        store.insert(k, v);
        Ok(())
    }
}

impl PermissionStore for MemoryStore {
    fn permit(&self, id: String, user: String, permission: Permission) -> CQRLResult<bool> {
        let store = self.permission_store.read().map_err(|_| errors::CQRLError::Generic)?;
        match store.get(&id) {
            Some(permissions) => {
                match permissions.get(&user) {
                    Some(p) => Ok(p.contains(&permission)),
                    None => Ok(false),
                }
            }
            None => Ok(false),
        }
    }

    fn grant(&mut self, id: String, user: String, permission: Permission) -> CQRLResult<()> {
        let mut store = self.permission_store.write().map_err(|_| errors::CQRLError::StoreError)?;
        let permissions = store.entry(id).or_insert(HashMap::new());
        let user_permissions = permissions.entry(user).or_insert(Vec::new());
        if !user_permissions.contains(&permission) {
            user_permissions.push(permission);
        }
        Ok(())
    }

    fn revoke(&mut self, id: String, user: String, permission: Permission) -> CQRLResult<()> {
        let mut store = self.permission_store.write().map_err(|_| errors::CQRLError::StoreError)?;
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
    use crate::Store;
    use serde_json::json;

    #[test]
    fn test_store_object() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_object(
                    "value".to_string(),
                    json!({
                        "test": "value"
                    })
                )
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

    #[test]
    fn test_store_operation() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_operation(
                    "value".to_string(),
                    json!({
                        "test": "value"
                    })
                )
                .is_ok(),
            true
        );

        assert_eq!(store.operation_store.read().unwrap().len(), 1);
        assert_eq!(
            store.operation_store.read().unwrap().get(&"value".to_string()),
            Some(&json!({
                "test": "value"
            }))
        );
    }

    #[test]
    fn end_to_end() {
        let mut store = MemoryStore::new();

        assert_eq!(
            store
                .store_object(
                    "value".to_string(),
                    json!({
                        "test": "value"
                    })
                )
                .is_ok(),
            true
        );

        assert_eq!(store.object_store.read().unwrap().len(), 1);
        let mut result = store.get_object(Some("value".to_string()));
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(
            value,
            json!({
                "test": "value"
            })
        );
        result = store.get_object(Some("invalid".to_string()));
        assert!(result.is_err());
    }
}

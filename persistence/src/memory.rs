use errors::CQRLResult;
use indexmap::IndexMap;
use serde_json::Value;

use crate::Store;

#[derive(Clone)]
pub struct MemoryStore {
    object_store: IndexMap<String, Value>,
    operation_store: IndexMap<String, Value>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            object_store: IndexMap::new(),
            operation_store: IndexMap::new(),
        }
    }
}

impl Store for MemoryStore {
    fn get_object(&self, k: Option<String>) -> CQRLResult<Value> {
        match k {
            Some(key) => match self.object_store.get(&key) {
                Some(v) => Ok(v.clone()),
                None => Err(errors::CQRLError::Generic),
            }
            None => Ok(Value::Array(vec![])),
        }
    }

    fn store_object(&mut self, k: String, v: Value) -> CQRLResult<()> {
        self.object_store.insert(k, v);
        Ok(())
    }

    fn store_operation(&mut self, k: String, v: Value) -> CQRLResult<()> {
        self.operation_store.insert(k, v);
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

        assert_eq!(store.object_store.len(), 1);
        assert_eq!(
            store.object_store.get(&"value".to_string()),
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

        assert_eq!(store.operation_store.len(), 1);
        assert_eq!(
            store.operation_store.get(&"value".to_string()),
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

        assert_eq!(store.object_store.len(), 1);
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

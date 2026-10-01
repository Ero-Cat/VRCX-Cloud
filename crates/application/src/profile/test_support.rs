use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;
use vrcx_0_application_core::Result;
use vrcx_0_contracts::{resolve_config_key, ConfigMutation, ConfigReadEntry, ConfigWriteEntry};

use super::ProfileConfigStore;

#[derive(Default)]
pub(super) struct MemoryProfileConfigStore {
    values: Mutex<HashMap<String, String>>,
    storage: Mutex<HashMap<String, String>>,
}

impl ProfileConfigStore for MemoryProfileConfigStore {
    fn get_raw(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .values
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&resolve_config_key(key))
            .cloned())
    }

    fn get_bool(&self, key: &str, default_value: bool) -> Result<bool> {
        Ok(self
            .get_raw(key)?
            .and_then(|value| value.parse().ok())
            .unwrap_or(default_value))
    }

    fn get_string(&self, key: &str, default_value: &str) -> Result<String> {
        Ok(self
            .get_raw(key)?
            .unwrap_or_else(|| default_value.to_string()))
    }

    fn get_json(&self, key: &str, default_value: Value) -> Result<Value> {
        Ok(self
            .get_raw(key)?
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or(default_value))
    }

    fn apply_mutations(&self, mutations: Vec<ConfigMutation>) -> Result<()> {
        let mut values = self
            .values
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for mutation in mutations {
            let key = resolve_config_key(&mutation.key);
            if let Some(value) = mutation.value {
                values.insert(key, value);
            } else {
                values.remove(&key);
            }
        }
        Ok(())
    }

    fn list_values(&self) -> Result<Vec<ConfigReadEntry>> {
        Ok(self
            .values
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .iter()
            .map(|(key, value)| ConfigReadEntry {
                key: key.clone(),
                value: value.clone(),
            })
            .collect())
    }

    fn set_values(&self, entries: Vec<ConfigWriteEntry>) -> Result<()> {
        let mut values = self
            .values
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for entry in entries {
            values.insert(resolve_config_key(&entry.key), entry.value);
        }
        Ok(())
    }

    fn remove_value(&self, key: String) -> Result<i64> {
        Ok(i64::from(
            self.values
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(&resolve_config_key(&key))
                .is_some(),
        ))
    }

    fn storage_get(&self, key: &str) -> Option<String> {
        self.storage
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(key)
            .cloned()
    }

    fn storage_set(&self, key: String, value: String) {
        self.storage
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(key, value);
    }

    fn storage_save(&self) -> Result<()> {
        Ok(())
    }
}

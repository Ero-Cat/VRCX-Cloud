use serde_json::Value;
use vrcx_0_application_core::Result;
use vrcx_0_contracts::{ConfigMutation, ConfigReadEntry, ConfigWriteEntry};

pub trait ProfileConfigStore: Send + Sync {
    fn get_raw(&self, key: &str) -> Result<Option<String>>;
    fn get_bool(&self, key: &str, default_value: bool) -> Result<bool>;
    fn get_string(&self, key: &str, default_value: &str) -> Result<String>;
    fn get_json(&self, key: &str, default_value: Value) -> Result<Value>;
    fn apply_mutations(&self, mutations: Vec<ConfigMutation>) -> Result<()>;
    fn list_values(&self) -> Result<Vec<ConfigReadEntry>>;
    fn set_values(&self, entries: Vec<ConfigWriteEntry>) -> Result<()>;
    fn remove_value(&self, key: String) -> Result<i64>;
    fn storage_get(&self, key: &str) -> Option<String>;
    fn storage_set(&self, key: String, value: String);
    fn storage_save(&self) -> Result<()>;
}

pub use vrcx_0_contracts::{resolve_config_key, ConfigMutation, ConfigReadEntry, ConfigWriteEntry};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConfigKey(String);

impl ConfigKey {
    pub fn new(key: &str) -> Self {
        Self(resolve_config_key(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ConfigKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ConfigKey {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

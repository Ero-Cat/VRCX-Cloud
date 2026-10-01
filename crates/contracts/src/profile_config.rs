use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigWriteEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigReadEntry {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigMutation {
    pub key: String,
    pub value: Option<String>,
}

impl ConfigMutation {
    pub fn set(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
        }
    }

    pub fn remove(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }
}

pub fn resolve_config_key(key: &str) -> String {
    let key = key.trim();
    if let Some(rest) = key.strip_prefix("config:") {
        return format!("config:{}", rest.to_lowercase());
    }

    let stripped = key.strip_prefix("VRCX_").unwrap_or(key);
    format!("config:vrcx_{}", stripped.to_lowercase())
}

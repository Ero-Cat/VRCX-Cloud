use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
pub struct OverlaySurfaceId(String);

impl OverlaySurfaceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub const MAIN_SURFACE_ID: &str = "main";

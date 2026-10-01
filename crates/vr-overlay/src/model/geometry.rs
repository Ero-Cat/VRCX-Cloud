use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct OverlaySize {
    pub width: u32,
    pub height: u32,
}

impl OverlaySize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

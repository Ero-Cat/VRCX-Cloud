//! Realtime event payload types for the game domain.
//!
//! Trimmed for the server fork: only the now-playing snapshot the
//! presence-automation facts consume remains; ingest-side events and the
//! game-client event stream are desktop-only.

#[derive(Clone, Debug, PartialEq, serde::Serialize, specta::Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingSnapshot {
    pub url: String,
    pub name: String,
    pub source: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub thumbnail_url: String,
    pub length: i64,
    pub position: i64,
    pub started_at: Option<String>,
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_id: Option<String>,
    pub updated_at: Option<String>,
}
impl NowPlayingSnapshot {
    pub fn has_content(&self) -> bool {
        !self.url.trim().is_empty() || !self.name.trim().is_empty()
    }
}

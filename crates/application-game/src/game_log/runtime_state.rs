//! Game-log runtime state primitives shared by queries and presence
//! automation on the server.

use serde::Serialize;

pub use vrcx_0_core::location::world_id_from_location;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PlayerState {
    pub user_id: String,
    pub display_name: String,
    pub join_time_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSnapshot {
    pub ready: bool,
    pub has_player_events: bool,
    pub location: String,
    pub world_name: String,
    pub destination: String,
    pub started_at: String,
    pub players: Vec<PlayerState>,
}

pub fn parse_event_time_ms(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

pub fn duration_ms(started_at: Option<i64>, stopped_at: Option<i64>) -> i64 {
    match (started_at, stopped_at) {
        (Some(started_at), Some(stopped_at)) if stopped_at >= started_at => stopped_at - started_at,
        _ => 0,
    }
}

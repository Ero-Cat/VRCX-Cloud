use serde::{Deserialize, Serialize};

use vrcx_0_core::OwnerId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisitTimelineInput {
    pub owner_user_id: OwnerId,
    pub at: String,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisitTimelineOutput {
    pub visit: Option<VisitRow>,
    pub roster: Vec<VisitRosterRow>,
    pub people_observed: usize,
    pub truncated: bool,
    pub summary: String,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisitRow {
    pub world_id: String,
    pub world_name: String,
    pub location: String,
    pub joined_at: String,
    pub left_at: Option<String>,
    pub stay_minutes: i64,
    pub in_progress: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisitRosterRow {
    pub user_id: String,
    pub display_name: String,
    pub is_friend: bool,
    pub first_joined_at: Option<String>,
    pub last_left_at: Option<String>,
    pub shared_minutes: i64,
    pub stints: Vec<VisitStint>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VisitStint {
    pub joined_at: Option<String>,
    pub left_at: Option<String>,
}

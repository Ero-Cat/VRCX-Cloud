use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::TimeWindow;
use vrcx_0_core::OwnerId;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum InviteDirection {
    Received,
    Sent,
    #[default]
    Both,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InviteHistoryInput {
    pub owner_user_id: OwnerId,
    pub time_window: TimeWindow,
    #[serde(default)]
    pub direction: InviteDirection,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InviteHistoryOutput {
    pub rows: Vec<InviteHistoryRow>,
    pub summary: String,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InviteHistoryRow {
    pub user_id: String,
    pub display_name: String,
    pub direction: InviteDirection,
    pub total_count: i64,
    pub last_invite_at: String,
    pub types: BTreeMap<String, i64>,
}

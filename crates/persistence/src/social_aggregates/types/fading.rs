use crate::ownership::OwnerId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FadingFriendsInput {
    pub owner_user_id: OwnerId,
    pub prior_from: String,
    pub pivot: String,
    pub now: String,
    #[serde(default)]
    pub min_prior_minutes: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FadingFriendsOutput {
    pub rows: Vec<FadingFriendRow>,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FadingFriendRow {
    pub user_id: String,
    pub display_name: String,
    pub prior_minutes: i64,
    pub recent_minutes: i64,
    pub prior_co_days: usize,
    pub recent_co_days: usize,
    pub drop_percent: i64,
    pub last_seen_together: String,
}

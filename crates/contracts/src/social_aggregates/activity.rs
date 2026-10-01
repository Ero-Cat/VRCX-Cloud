use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ActivityBucket, TimeWindow};
use vrcx_0_core::OwnerId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendActivityPatternInput {
    pub owner_user_id: OwnerId,
    #[serde(default)]
    pub user_id: Option<String>,
    pub time_window: TimeWindow,
    #[serde(default)]
    pub bucket: ActivityBucket,
    #[serde(default)]
    pub utc_offset_minutes: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendActivityPatternOutput {
    pub rows: Vec<FriendActivityPatternRow>,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendActivityPatternRow {
    pub user_id: String,
    pub display_name: String,
    pub buckets: BTreeMap<String, i64>,
    pub typical_online_window: String,
}

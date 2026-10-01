use serde::{Deserialize, Serialize};
use vrcx_0_core::OwnerId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolveUserInput {
    pub owner_user_id: OwnerId,
    pub name_query: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolveUserOutput {
    pub rows: Vec<ResolvedUserRow>,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedUserRow {
    pub user_id: String,
    pub display_name: String,
    pub matched_name: String,
    pub is_friend: bool,
    pub encounter_count: i64,
    pub last_seen: String,
}

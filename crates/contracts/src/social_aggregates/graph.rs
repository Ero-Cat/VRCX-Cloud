use serde::{Deserialize, Serialize};
use vrcx_0_core::OwnerId;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocialGraphInput {
    pub owner_user_id: OwnerId,
    #[serde(default)]
    pub user_id: Option<String>,
    pub depth: u8,
    #[serde(default)]
    pub max_nodes: Option<i64>,
    #[serde(default)]
    pub max_edges: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocialGraphOutput {
    pub nodes: Vec<SocialGraphNode>,
    pub edges: Vec<SocialGraphEdge>,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub truncated: bool,
    pub fetched_friends: usize,
    pub opted_out_friends: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_fetched_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_fetched_at: Option<String>,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocialGraphNode {
    pub user_id: String,
    pub display_name: String,
    pub is_friend: bool,
    pub connection_degree: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocialGraphEdge {
    pub source_user_id: String,
    pub target_user_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendCirclesInput {
    pub owner_user_id: OwnerId,
    #[serde(default)]
    pub max_circles: Option<i64>,
    #[serde(default)]
    pub max_members_per_circle: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendCirclesOutput {
    pub circles: Vec<FriendCircleRow>,
    pub circle_count: usize,
    pub isolated_friend_count: usize,
    pub friends_analyzed: usize,
    pub summary: String,
    pub caveats: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendCircleRow {
    pub members: Vec<String>,
    pub member_count: usize,
    pub sample_pairs: Vec<FriendCirclePair>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FriendCirclePair {
    pub a: String,
    pub b: String,
}

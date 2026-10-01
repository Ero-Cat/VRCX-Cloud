use serde::{Deserialize, Serialize};
use vrcx_0_core::ReleaseStatus;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldCollectionCreatePayload {
    pub schema: i64,
    pub owner_hint: String,
    pub title: String,
    pub listed: bool,
    pub access: String,
    pub author_name: String,
    pub updated_at: i64,
    pub worlds: Vec<WorldCollectionPayloadWorld>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldCollectionPayloadWorld {
    pub world_id: String,
    pub author_id: String,
    pub name: String,
    pub author_name: String,
    pub created_at: String,
    pub image_url: String,
    pub description: String,
    pub release_status: ReleaseStatus,
    pub thumbnail_image_url: String,
    pub comment: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldCollectionSkippedWorld {
    pub world_id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldCollectionCreateResponse {
    pub id: String,
    #[serde(default)]
    pub skipped_worlds: Vec<WorldCollectionSkippedWorld>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldCollectionTokenMintRequest {
    pub owner_hint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldOpenRegisterPayload {
    pub schema: i64,
    pub owner_hint: String,
    pub world: WorldOpenRegisterWorld,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldOpenRegisterWorld {
    pub world_id: String,
    pub author_id: String,
    pub name: String,
    pub author_name: String,
    pub created_at: String,
    pub image_url: String,
    pub thumbnail_image_url: String,
    pub description: String,
    pub release_status: ReleaseStatus,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WorldCollectionTokenMintResponse {
    pub token: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct WorldCollectionSnapshotWorld {
    pub world_id: String,
    pub name: String,
    pub author_name: String,
    pub image_url: String,
    pub description: String,
    pub comment: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct WorldCollectionSnapshotResponse {
    pub id: String,
    pub title: String,
    pub note: Option<String>,
    pub author_name: String,
    pub author_profile: Option<String>,
    pub category: Option<String>,
    pub listed: bool,
    pub updated_at: i64,
    pub worlds: Vec<WorldCollectionSnapshotWorld>,
}

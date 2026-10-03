use serde::{Deserialize, Serialize};

// ── Bio history (diff view source) ──

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FeedBioHistoryQueryInput {
    pub user_id: String,
    pub target_user_id: String,
    #[serde(default)]
    pub date_from: String,
    #[serde(default)]
    pub date_to: String,
    #[serde(default)]
    pub limit: i64,
    /// Consecutive changes closer than this many hours collapse into one
    /// diff entry (None = default 24h, Some(0) disables merging).
    #[serde(default)]
    pub merge_hours: Option<i64>,
}

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FeedBioHistoryRow {
    pub row_id: i64,
    /// Time of the newest change inside the merge group.
    pub created_at: String,
    /// Time of the oldest change inside the merge group.
    pub first_created_at: String,
    pub display_name: Option<String>,
    pub previous_bio: Option<String>,
    pub bio: Option<String>,
    pub merged_changes: i64,
}

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FeedBioHistoryOutput {
    /// Newest first.
    pub rows: Vec<FeedBioHistoryRow>,
}

/// jirai's recordBioSnapshot: when a dialog opens with the current bio,
/// diff it against the last recorded one and append a history row when it
/// changed. The first record is the baseline.
#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FeedBioSnapshotRecordInput {
    pub user_id: String,
    pub target_user_id: String,
    pub bio: String,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FeedBioSnapshotRecordOutput {
    pub changed: bool,
    pub first_record: bool,
}

// ── Status light distribution ──

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusStatsTotal {
    /// Normalized status key: active / join me / ask me / busy / offline.
    pub status: String,
    pub minutes: i64,
    /// Share of tracked online time, 0..=1.
    pub share: f64,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusStatsDailyBucket {
    /// Local calendar day (yyyy-mm-dd, shifted by utcOffsetMinutes).
    pub day: String,
    pub status: String,
    pub minutes: i64,
}

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusStatsViewInput {
    pub owner_user_id: String,
    pub target_user_id: String,
    #[serde(default)]
    pub range_days: i64,
    #[serde(default)]
    pub utc_offset_minutes: i64,
    #[serde(default)]
    pub now_ms: i64,
    /// Self variant derives presence from the owner's own game log instead
    /// of the friend presence feed.
    #[serde(default)]
    pub is_self: bool,
}

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusStatsViewOutput {
    pub range_days: i64,
    pub totals: Vec<StatusStatsTotal>,
    /// Sorted by day ascending.
    pub daily: Vec<StatusStatsDailyBucket>,
    /// Total online minutes with a known status inside the range.
    pub tracked_minutes: i64,
    pub has_any_data: bool,
    pub built_at: String,
}

// ── Two-person relationship (game-log based, VRCX-jirai parity) ──

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoPersonRelationshipQueryInput {
    pub owner_user_id: String,
    pub user_id_a: String,
    pub user_id_b: String,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoPersonOverlapRow {
    pub location: String,
    pub friend_a_leave: String,
    pub friend_a_time: i64,
    pub friend_b_leave: String,
    pub friend_b_time: i64,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoPersonSelfPresenceRow {
    pub location: String,
    pub self_leave: String,
    pub self_time: i64,
}

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TwoPersonRelationshipOutput {
    /// Overlapping OnPlayerLeft rows between the two users, deduplicated.
    pub rows: Vec<TwoPersonOverlapRow>,
    /// The owner's own sessions in those locations.
    pub self_presence: Vec<TwoPersonSelfPresenceRow>,
    /// Peak concurrent occupancy per location (sweep over every
    /// OnPlayerLeft record, not just the two selected users).
    pub max_player_counts: Vec<(String, i64)>,
}

// ── Relationship timeline (game-log based) ──

#[derive(Clone, Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTimelineRowsInput {
    pub owner_user_id: String,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTimelineDayRow {
    pub user_id: String,
    pub display_name: String,
    /// Local-less calendar day (yyyy-mm-dd, from created_at in UTC —
    /// matching the desktop recorder's timestamps).
    pub day: String,
    pub total_time_ms: i64,
    pub join_count: i64,
}

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTimelineRowsOutput {
    pub rows: Vec<RelationshipTimelineDayRow>,
}

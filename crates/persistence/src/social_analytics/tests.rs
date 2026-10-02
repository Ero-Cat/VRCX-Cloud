use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;

use crate::common::ParamsBuilder;
use crate::database::DatabaseService;
use crate::realtime::{ensure_realtime_tables, normalize_user_table_prefix};

use super::bio_history::feed_bio_history_query;
use super::status_stats::status_stats_view;
use super::tracks::presence_intervals_for_user;
use vrcx_0_contracts::social_analytics::{FeedBioHistoryQueryInput, StatusStatsViewInput};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "vrcx-0-social-analytics-{name}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn test_db(name: &str) -> DatabaseService {
    let dir = TestDir::new(name);
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    // Keep the temp dir alive for the process: the SQLite handle owns files
    // inside it and each test uses a unique nonce anyway.
    std::mem::forget(dir);
    db
}

const OWNER: &str = "usr_owner";

fn prefix() -> String {
    normalize_user_table_prefix(OWNER).unwrap()
}

fn seed_online_offline(
    db: &DatabaseService,
    (created_at, user_id, kind, location): (&str, &str, &str, &str),
) {
    let user_prefix = prefix();
    ensure_realtime_tables(db, &user_prefix).unwrap();
    db.execute_non_query(
        &format!(
            "INSERT INTO {user_prefix}_feed_online_offline (created_at, user_id, display_name, type, location, world_name, time, group_name)
             VALUES (@created_at, @user_id, 'Friend', @type, @location, 'World', 0, '')"
        ),
        &ParamsBuilder::new()
            .set("created_at", created_at)
            .set("user_id", user_id)
            .set("type", kind)
            .set("location", location)
            .build(),
    )
    .unwrap();
}

fn seed_gps(
    db: &DatabaseService,
    (created_at, user_id, location, previous_location): (&str, &str, &str, &str),
) {
    let user_prefix = prefix();
    ensure_realtime_tables(db, &user_prefix).unwrap();
    db.execute_non_query(
        &format!(
            "INSERT INTO {user_prefix}_feed_gps (created_at, user_id, display_name, location, world_name, previous_location, time, group_name)
             VALUES (@created_at, @user_id, 'Friend', @location, 'World', @previous_location, 0, '')"
        ),
        &ParamsBuilder::new()
            .set("created_at", created_at)
            .set("user_id", user_id)
            .set("location", location)
            .set("previous_location", previous_location)
            .build(),
    )
    .unwrap();
}

fn seed_status(db: &DatabaseService, (created_at, user_id, status): (&str, &str, &str)) {
    let user_prefix = prefix();
    ensure_realtime_tables(db, &user_prefix).unwrap();
    db.execute_non_query(
        &format!(
            "INSERT INTO {user_prefix}_feed_status (created_at, user_id, display_name, status, status_description, previous_status, previous_status_description)
             VALUES (@created_at, @user_id, 'Friend', @status, '', 'active', '')"
        ),
        &ParamsBuilder::new()
            .set("created_at", created_at)
            .set("user_id", user_id)
            .set("status", status)
            .build(),
    )
    .unwrap();
}

fn seed_bio(
    db: &DatabaseService,
    (created_at, user_id, bio, previous_bio): (&str, &str, &str, &str),
) {
    let user_prefix = prefix();
    ensure_realtime_tables(db, &user_prefix).unwrap();
    db.execute_non_query(
        &format!(
            "INSERT INTO {user_prefix}_feed_bio (created_at, user_id, display_name, bio, previous_bio)
             VALUES (@created_at, @user_id, 'Friend', @bio, @previous_bio)"
        ),
        &ParamsBuilder::new()
            .set("created_at", created_at)
            .set("user_id", user_id)
            .set("bio", bio)
            .set("previous_bio", previous_bio)
            .build(),
    )
    .unwrap();
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn iso_after(hours: i64) -> String {
    let ms = now_ms() + hours * 60 * 60 * 1000 - 48 * 60 * 60 * 1000;
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

// ── tracks ──

#[test]
fn presence_intervals_pair_online_offline() {
    let db = test_db("presence");
    seed_online_offline(&db, (&iso_after(0), "usr_a", "Online", "wrld_1:1"));
    seed_online_offline(&db, (&iso_after(1), "usr_a", "Offline", ""));
    seed_online_offline(&db, (&iso_after(3), "usr_a", "Online", "wrld_2:2"));
    let intervals = presence_intervals_for_user(&db, &prefix(), "usr_a", "", now_ms()).unwrap();
    assert_eq!(intervals.len(), 2);
}

// ── bio history ──

#[test]
fn bio_history_merges_changes_within_the_window() {
    let db = test_db("bio-merge");
    seed_bio(&db, (&iso_after(0), "usr_a", "v1", "v0"));
    seed_bio(&db, (&iso_after(1), "usr_a", "v2", "v1"));
    seed_bio(&db, (&iso_after(2), "usr_a", "v3", "v2"));
    let output = feed_bio_history_query(
        &db,
        FeedBioHistoryQueryInput {
            user_id: OWNER.into(),
            target_user_id: "usr_a".into(),
            date_from: String::new(),
            date_to: String::new(),
            limit: 0,
            merge_hours: Some(24),
        },
    )
    .unwrap();
    assert_eq!(output.rows.len(), 1);
    let row = &output.rows[0];
    assert_eq!(row.previous_bio.as_deref(), Some("v0"));
    assert_eq!(row.bio.as_deref(), Some("v3"));
    assert_eq!(row.merged_changes, 3);
}

#[test]
fn bio_history_keeps_far_apart_changes_separate() {
    let db = test_db("bio-split");
    seed_bio(&db, (&iso_after(0), "usr_a", "v1", "v0"));
    seed_bio(&db, (&iso_after(30), "usr_a", "v2", "v1"));
    let output = feed_bio_history_query(
        &db,
        FeedBioHistoryQueryInput {
            user_id: OWNER.into(),
            target_user_id: "usr_a".into(),
            date_from: String::new(),
            date_to: String::new(),
            limit: 0,
            merge_hours: Some(24),
        },
    )
    .unwrap();
    assert_eq!(output.rows.len(), 2);
    // Newest first.
    assert_eq!(output.rows[0].bio.as_deref(), Some("v2"));
}

// ── status stats ──

#[test]
fn status_stats_buckets_status_time_inside_presence() {
    let db = test_db("status-stats");
    // One string per instant: seeding Online and the first status at the
    // exact same timestamp keeps the minute arithmetic deterministic.
    let t0 = iso_after(0);
    let t1 = iso_after(1);
    let t2 = iso_after(2);
    seed_online_offline(&db, (&t0, "usr_a", "Online", "wrld_1:1"));
    seed_status(&db, (&t0, "usr_a", "join me"));
    seed_status(&db, (&t1, "usr_a", "busy"));
    seed_online_offline(&db, (&t2, "usr_a", "Offline", ""));
    let output = status_stats_view(
        &db,
        StatusStatsViewInput {
            owner_user_id: OWNER.into(),
            target_user_id: "usr_a".into(),
            range_days: 7,
            utc_offset_minutes: 0,
            now_ms: now_ms(),
            is_self: false,
        },
    )
    .unwrap();
    assert!(output.has_any_data);
    let join_me = output
        .totals
        .iter()
        .find(|total| total.status == "join me")
        .unwrap();
    let busy = output
        .totals
        .iter()
        .find(|total| total.status == "busy")
        .unwrap();
    assert_eq!(join_me.minutes, 60);
    assert_eq!(busy.minutes, 60);
    assert_eq!(output.tracked_minutes, 120);
    assert!((join_me.share - 0.5).abs() < 1e-9);
    assert!(output.daily.len() >= 2);
}

// ── mutual instances ──

#[test]
fn bio_history_ignores_other_users() {
    let db = test_db("bio-scope");
    seed_bio(&db, (&iso_after(0), "usr_other", "x", "y"));
    let output = feed_bio_history_query(
        &db,
        FeedBioHistoryQueryInput {
            user_id: OWNER.into(),
            target_user_id: "usr_a".into(),
            date_from: String::new(),
            date_to: String::new(),
            limit: 0,
            merge_hours: None,
        },
    )
    .unwrap();
    assert!(output.rows.is_empty());
    let _ = Value::Null;
}

fn seed_join_leave(
    db: &DatabaseService,
    (created_at, user_id, display_name, location, time): (&str, &str, &str, &str, i64),
) {
    crate::game_log::ensure_game_log_tables(db).unwrap();
    let owner_row_id =
        crate::ownership::owner_id_get_or_insert(db, &vrcx_0_core::OwnerId::new(OWNER)).unwrap();
    db.execute_non_query(
        "INSERT INTO gamelog_join_leave (created_at, type, display_name, location, user_id, time, owner_id)
         VALUES (@created_at, 'OnPlayerLeft', @display_name, @location, @user_id, @time, @owner_id)",
        &{
            let mut params = HashMap::new();
            params.insert("@created_at".into(), Value::String(created_at.to_string()));
            params.insert("@user_id".into(), Value::String(user_id.to_string()));
            params.insert("@display_name".into(), Value::String(display_name.to_string()));
            params.insert("@location".into(), Value::String(location.to_string()));
            params.insert("@time".into(), Value::from(time));
            params.insert("@owner_id".into(), Value::from(owner_row_id.value()));
            params
        },
    )
    .unwrap();
}

#[test]
fn two_person_relationship_finds_overlapping_stays() {
    let db = test_db("two-person");
    let t0 = iso_after(0);
    let t1 = iso_after(1);
    let t2 = iso_after(2);
    // A stays 10:00–12:00, B stays 11:00–12:00 → one hour of overlap.
    seed_join_leave(&db, (&t2, "usr_a", "A", "wrld_1:1", 2 * 60 * 60 * 1000));
    seed_join_leave(&db, (&t2, "usr_b", "B", "wrld_1:1", 60 * 60 * 1000));
    seed_join_leave(&db, (&t1, "usr_c", "C", "wrld_1:1", 30 * 60 * 1000));
    let output = super::two_person_relationship_query(&db, OWNER, "usr_a", "usr_b").unwrap();
    assert_eq!(output.rows.len(), 1);
    assert_eq!(output.rows[0].location, "wrld_1:1");
    // Peak occupancy sweeps every player: A overlaps B at 11:00-12:00,
    // and C hands over to B exactly at 11:00 (exits sort before enters).
    assert_eq!(
        output
            .max_player_counts
            .iter()
            .find(|(location, _)| location == "wrld_1:1")
            .map(|(_, count)| *count),
        Some(2)
    );
    // Empty result short-circuits without self-presence queries.
    assert_eq!(
        super::two_person_relationship_query(&db, OWNER, "usr_a", "usr_c")
            .unwrap()
            .rows
            .len(),
        1
    );
}

#[test]
fn two_person_relationship_excludes_private_and_zero_time() {
    let db = test_db("two-person-guard");
    let t1 = iso_after(1);
    let t2 = iso_after(2);
    seed_join_leave(&db, (&t2, "usr_a", "A", "private", 60 * 60 * 1000));
    seed_join_leave(&db, (&t1, "usr_b", "B", "private", 60 * 60 * 1000));
    seed_join_leave(&db, (&t1, "usr_a", "A", "wrld_2:2", 0));
    seed_join_leave(&db, (&t2, "usr_b", "B", "wrld_2:2", 0));
    let output = super::two_person_relationship_query(&db, OWNER, "usr_a", "usr_b").unwrap();
    assert!(output.rows.is_empty());
}

#[test]
fn relationship_timeline_rows_group_by_user_and_day() {
    let db = test_db("timeline-rows");
    seed_join_leave(
        &db,
        (&iso_after(0), "usr_a", "A", "wrld_1:1", 30 * 60 * 1000),
    );
    seed_join_leave(
        &db,
        (&iso_after(0), "usr_a", "A", "wrld_2:2", 15 * 60 * 1000),
    );
    seed_join_leave(
        &db,
        (&iso_after(24), "usr_b", "B", "wrld_1:1", 60 * 60 * 1000),
    );
    // The owner's own rows never appear.
    seed_join_leave(
        &db,
        (&iso_after(0), OWNER, "Self", "wrld_1:1", 60 * 60 * 1000),
    );
    let output = super::relationship_timeline_rows(
        &db,
        vrcx_0_contracts::social_analytics::RelationshipTimelineRowsInput {
            owner_user_id: OWNER.into(),
        },
    )
    .unwrap();
    let by_user: Vec<(String, i64, i64)> = output
        .rows
        .iter()
        .map(|row| (row.user_id.clone(), row.total_time_ms, row.join_count))
        .collect();
    assert_eq!(by_user.len(), 2);
    assert!(by_user.contains(&("usr_a".into(), 45 * 60 * 1000, 2)));
    assert!(by_user.contains(&("usr_b".into(), 60 * 60 * 1000, 1)));
}

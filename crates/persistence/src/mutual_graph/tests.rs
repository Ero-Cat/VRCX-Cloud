use std::path::PathBuf;

use super::*;

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
            "vrcx-0-mutual-graph-{name}-{}-{nonce}",
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

fn entry(friend_id: &str, mutual_ids: &[&str]) -> MutualGraphSnapshotEntryInput {
    MutualGraphSnapshotEntryInput {
        friend_id: friend_id.into(),
        mutual_ids: mutual_ids.iter().map(|id| (*id).into()).collect(),
    }
}

fn meta(friend_id: &str, opted_out: bool) -> MutualGraphMetaInput {
    MutualGraphMetaInput {
        friend_id: friend_id.into(),
        last_fetched_at: "2026-07-21T12:00:00Z".into(),
        opted_out,
        total_count: None,
    }
}

#[test]
fn full_snapshot_commit_removes_opted_out_nodes_that_are_no_longer_friends() {
    let dir = TestDir::new("remove-stale-opt-out");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    let user_id = "usr_self".to_string();

    mutual_graph_snapshot_commit(
        &db,
        user_id.clone(),
        vec![entry("usr_old", &["usr_mutual_old"])],
        vec![meta("usr_old", false)],
    )
    .unwrap();
    mutual_graph_snapshot_commit(
        &db,
        user_id.clone(),
        Vec::new(),
        vec![meta("usr_old", true)],
    )
    .unwrap();
    mutual_graph_snapshot_commit(
        &db,
        user_id.clone(),
        vec![entry("usr_current", &["usr_mutual_current"])],
        vec![meta("usr_current", false)],
    )
    .unwrap();

    let snapshot = mutual_graph_snapshot_get(&db, user_id).unwrap();
    assert_eq!(snapshot.friend_ids, vec!["usr_current"]);
    assert_eq!(
        snapshot
            .links
            .iter()
            .map(|link| (link.friend_id.as_str(), link.mutual_id.as_str()))
            .collect::<Vec<_>>(),
        vec![("usr_current", "usr_mutual_current")]
    );
    assert_eq!(
        snapshot
            .meta
            .iter()
            .map(|entry| entry.friend_id.as_str())
            .collect::<Vec<_>>(),
        vec!["usr_current"]
    );
}

#[test]
fn friend_refresh_replaces_links_and_opt_out_preserves_the_last_snapshot() {
    let dir = TestDir::new("friend-refresh");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    let user_id = "usr_self".to_string();

    mutual_graph_friend_refresh_commit(
        &db,
        user_id.clone(),
        "usr_friend".into(),
        Some(vec!["usr_old".into()]),
        Some(1),
        false,
    )
    .unwrap();
    mutual_graph_friend_refresh_commit(
        &db,
        user_id.clone(),
        "usr_friend".into(),
        Some(vec!["usr_new".into()]),
        Some(2),
        false,
    )
    .unwrap();
    mutual_graph_friend_refresh_commit(&db, user_id.clone(), "usr_friend".into(), None, None, true)
        .unwrap();

    let snapshot = mutual_graph_snapshot_get(&db, user_id).unwrap();
    assert_eq!(snapshot.friend_ids, vec!["usr_friend"]);
    assert_eq!(
        snapshot
            .links
            .iter()
            .map(|link| (link.friend_id.as_str(), link.mutual_id.as_str()))
            .collect::<Vec<_>>(),
        vec![("usr_friend", "usr_new")]
    );
    assert_eq!(snapshot.meta.len(), 1);
    assert!(snapshot.meta[0].opted_out);
    assert!(!snapshot.meta[0].last_fetched_at.is_empty());
    assert_eq!(snapshot.meta[0].total_count, Some(2));
}

#[test]
fn manual_links_and_external_users_survive_snapshot_refresh() {
    let dir = TestDir::new("manual-extras");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    let user_id = "usr_self".to_string();

    mutual_graph_manual_link_add(
        &db,
        user_id.clone(),
        "usr_b".into(),
        "usr_a".into(),
        "met in a bar".into(),
    )
    .unwrap();
    mutual_graph_external_user_add(
        &db,
        user_id.clone(),
        "usr_outsider".into(),
        "Outsider".into(),
        String::new(),
    )
    .unwrap();

    // A later full-snapshot refresh must not wipe the user-drawn data.
    mutual_graph_snapshot_commit(
        &db,
        user_id.clone(),
        vec![entry("usr_a", &["usr_b"])],
        vec![meta("usr_a", false)],
    )
    .unwrap();

    let snapshot = mutual_graph_snapshot_get(&db, user_id.clone()).unwrap();
    assert_eq!(snapshot.manual_links.len(), 1);
    // The pair is stored canonically regardless of argument order.
    assert_eq!(snapshot.manual_links[0].friend_id, "usr_a");
    assert_eq!(snapshot.manual_links[0].mutual_id, "usr_b");
    assert_eq!(snapshot.manual_links[0].note, "met in a bar");
    assert_eq!(snapshot.external_users.len(), 1);
    assert_eq!(snapshot.external_users[0].user_id, "usr_outsider");
    assert_eq!(snapshot.external_users[0].display_name, "Outsider");

    mutual_graph_manual_link_remove(&db, user_id.clone(), "usr_b".into(), "usr_a".into()).unwrap();
    mutual_graph_external_user_remove(&db, user_id, "usr_outsider".into()).unwrap();
    let snapshot = mutual_graph_snapshot_get(&db, "usr_self".to_string()).unwrap();
    assert!(snapshot.manual_links.is_empty());
    assert!(snapshot.external_users.is_empty());
}

#[test]
fn manual_link_add_ignores_self_links_and_duplicates() {
    let dir = TestDir::new("manual-link-guard");
    let db = DatabaseService::new(&dir.path.join("VRCX-0.sqlite3")).unwrap();
    let user_id = "usr_self".to_string();

    mutual_graph_manual_link_add(
        &db,
        user_id.clone(),
        "usr_a".into(),
        "usr_a".into(),
        String::new(),
    )
    .unwrap();
    mutual_graph_manual_link_add(
        &db,
        user_id.clone(),
        "usr_a".into(),
        "usr_b".into(),
        "first".into(),
    )
    .unwrap();
    mutual_graph_manual_link_add(
        &db,
        user_id.clone(),
        "usr_b".into(),
        "usr_a".into(),
        "second".into(),
    )
    .unwrap();

    let snapshot = mutual_graph_snapshot_get(&db, user_id).unwrap();
    assert_eq!(snapshot.manual_links.len(), 1);
    assert_eq!(snapshot.manual_links[0].note, "first");
}

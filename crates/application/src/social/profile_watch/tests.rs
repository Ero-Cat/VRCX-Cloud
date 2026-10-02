use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::json;
use vrcx_0_application_core::Result;
use vrcx_0_application_realtime::test_support::{
    feed_lookup_input, runtime_with_active_session, TestDir, TestRealtimeHostRuntime,
};
use vrcx_0_contracts::feed::{FeedFilter, FeedRowOutput, FeedRowsQueryInput};
use vrcx_0_core::OwnerId;

use super::*;

#[derive(Default)]
struct MemoryProfileWatchStore {
    snapshots: Mutex<HashMap<(String, String), ProfileWatchSeen>>,
    candidates: Mutex<Vec<String>>,
}

impl ProfileWatchStore for MemoryProfileWatchStore {
    fn next_stale(&self, _owner: &OwnerId, _checked_before: &str) -> Result<Option<String>> {
        Ok(self.candidates.lock().unwrap().first().cloned())
    }

    fn snapshot(&self, owner: &OwnerId, user_id: &str) -> Result<Option<ProfileWatchSeen>> {
        Ok(self
            .snapshots
            .lock()
            .unwrap()
            .get(&(owner.as_str().to_string(), user_id.to_string()))
            .cloned())
    }

    fn record(
        &self,
        owner: &OwnerId,
        user_id: &str,
        observation: &ProfileWatchObservation,
        polled_at: &str,
    ) -> Result<()> {
        let _ = polled_at;
        self.snapshots.lock().unwrap().insert(
            (owner.as_str().to_string(), user_id.to_string()),
            ProfileWatchSeen {
                display_name: observation.display_name.clone(),
                status: observation.status.clone(),
                status_description: observation.status_description.clone(),
                bio: observation.bio.clone(),
            },
        );
        Ok(())
    }

    fn mark_checked(&self, _owner: &OwnerId, _user_id: &str, _checked_at: &str) -> Result<()> {
        Ok(())
    }
}

fn observation(status: &str, bio: &str) -> ProfileWatchObservation {
    ProfileWatchObservation {
        user_id: "usr_outsider".into(),
        display_name: "Outsider".into(),
        status: status.into(),
        status_description: String::new(),
        bio: bio.into(),
    }
}

fn runtime_with_owner(name: &str) -> Result<(TestDir, TestRealtimeHostRuntime, OwnerId)> {
    let (dir, runtime, session) = runtime_with_active_session(name)?;
    Ok((dir, runtime, OwnerId::new(session.user_id)))
}

fn feed_rows(runtime: &TestRealtimeHostRuntime, owner: &OwnerId) -> Vec<FeedRowOutput> {
    runtime
        .store()
        .feed_rows(FeedRowsQueryInput {
            filters: vec![FeedFilter::Bio, FeedFilter::Status],
            ..feed_lookup_input(owner.as_str().into())
        })
        .unwrap()
}

#[test]
fn first_observation_is_a_baseline_without_publishing() {
    let (_dir, runtime, owner) = runtime_with_owner("watch-baseline").unwrap();
    let store = MemoryProfileWatchStore::default();
    let outcome = observe_profile_watch(
        &store,
        runtime.runtime(),
        &owner,
        &observation("active", "hello"),
        "2026-10-01T00:00:00Z",
    )
    .unwrap();
    assert_eq!(outcome, ProfileWatchOutcome::Baseline);
    assert!(feed_rows(&runtime, &owner).is_empty());
}

#[test]
fn changed_fields_publish_feed_entries_for_non_friends() {
    let (_dir, runtime, owner) = runtime_with_owner("watch-changed").unwrap();
    let store = MemoryProfileWatchStore::default();
    observe_profile_watch(
        &store,
        runtime.runtime(),
        &owner,
        &observation("active", "hello"),
        "2026-10-01T00:00:00Z",
    )
    .unwrap();

    let outcome = observe_profile_watch(
        &store,
        runtime.runtime(),
        &owner,
        &observation("join me", "world"),
        "2026-10-01T01:00:00Z",
    )
    .unwrap();
    assert_eq!(outcome, ProfileWatchOutcome::Changed { published: 2 });

    let rows = feed_rows(&runtime, &owner);
    assert_eq!(rows.len(), 2);
    let bio_row = rows
        .iter()
        .find(|row| row.r#type.as_deref() == Some("Bio"))
        .unwrap();
    assert_eq!(bio_row.user_id.as_deref(), Some("usr_outsider"));
    assert_eq!(bio_row.bio.as_deref(), Some("world"));
    assert_eq!(bio_row.previous_bio.as_deref(), Some("hello"));
    let status_row = rows
        .iter()
        .find(|row| row.r#type.as_deref() == Some("Status"))
        .unwrap();
    assert_eq!(status_row.status.as_deref(), Some("join me"));
    assert_eq!(status_row.previous_status.as_deref(), Some("active"));
}

#[test]
fn unchanged_observation_publishes_nothing() {
    let (_dir, runtime, owner) = runtime_with_owner("watch-unchanged").unwrap();
    let store = MemoryProfileWatchStore::default();
    observe_profile_watch(
        &store,
        runtime.runtime(),
        &owner,
        &observation("active", "hello"),
        "2026-10-01T00:00:00Z",
    )
    .unwrap();
    let outcome = observe_profile_watch(
        &store,
        runtime.runtime(),
        &owner,
        &observation("active", "hello"),
        "2026-10-01T01:00:00Z",
    )
    .unwrap();
    assert_eq!(outcome, ProfileWatchOutcome::Unchanged);
    assert!(feed_rows(&runtime, &owner).is_empty());
}

#[test]
fn observation_parses_profile_payloads() {
    let parsed = ProfileWatchObservation::from_profile(
        &json!({ "id": "usr_x", "displayName": "X", "status": "joinme", "statusDescription": "hi", "bio": " b " }),
    )
    .unwrap();
    assert_eq!(parsed.status, "joinme");
    assert_eq!(parsed.status_description, "hi");
    assert_eq!(parsed.bio, "b");
    assert!(ProfileWatchObservation::from_profile(&json!({ "id": "grp_x" })).is_none());
}

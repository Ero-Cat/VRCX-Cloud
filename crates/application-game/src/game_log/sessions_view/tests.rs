use crate::ports::TestGameStateStore;
use crate::GameStateStore;
use vrcx_0_contracts::game_log::{
    GameLogJoinLeaveEntry, GameLogLocationEntry, GameLogVideoPlayEntry, GameLogWriteBatch,
};

use super::*;

fn test_store(_name: &str) -> TestGameStateStore {
    TestGameStateStore::default()
}

fn location(
    created_at: &str,
    location: &str,
    world_id: &str,
    world_name: &str,
) -> GameLogLocationEntry {
    GameLogLocationEntry {
        created_at: created_at.to_string(),
        location: location.to_string(),
        world_id: world_id.to_string(),
        world_name: world_name.to_string(),
        time: 0,
        group_name: String::new(),
    }
}

fn join(
    created_at: &str,
    display_name: &str,
    location: &str,
    user_id: &str,
) -> GameLogJoinLeaveEntry {
    GameLogJoinLeaveEntry {
        created_at: created_at.to_string(),
        event_type: "OnPlayerJoined".to_string(),
        display_name: display_name.to_string(),
        location: location.to_string(),
        user_id: user_id.to_string(),
        world_name: String::new(),
        time: 0,
    }
}

fn leave(
    created_at: &str,
    display_name: &str,
    location: &str,
    user_id: &str,
    time: i64,
) -> GameLogJoinLeaveEntry {
    GameLogJoinLeaveEntry {
        created_at: created_at.to_string(),
        event_type: "OnPlayerLeft".to_string(),
        display_name: display_name.to_string(),
        location: location.to_string(),
        user_id: user_id.to_string(),
        world_name: String::new(),
        time,
    }
}

fn video(created_at: &str, url: &str, location: &str) -> GameLogVideoPlayEntry {
    GameLogVideoPlayEntry {
        created_at: created_at.to_string(),
        video_url: url.to_string(),
        video_name: "Clip".to_string(),
        video_id: String::new(),
        location: location.to_string(),
        display_name: String::new(),
        user_id: String::new(),
    }
}

fn write_rows(
    store: &TestGameStateStore,
    locations: Vec<GameLogLocationEntry>,
    join_leave: Vec<GameLogJoinLeaveEntry>,
    video_plays: Vec<GameLogVideoPlayEntry>,
) {
    let batch = GameLogWriteBatch {
        locations,
        join_leave,
        video_plays,
        ..Default::default()
    };
    store.write_game_log(&OwnerId::new(""), &batch).unwrap();
}

fn query(store: &TestGameStateStore, input: GameLogSessionsQueryInput) -> Vec<GameLogSessionDto> {
    game_log_sessions_query(store, &OwnerId::new(""), input).unwrap()
}

#[test]
fn returns_sessions_newest_first_with_video_merge() {
    let store = test_store("sessions-newest-first");
    write_rows(
        &store,
        vec![
            location("2026-01-01T10:00:00.000Z", "wrld_old:1", "wrld_old", "Old"),
            location("2026-01-01T11:00:00.000Z", "wrld_new:1", "wrld_new", "New"),
        ],
        vec![join("2026-01-01T10:00:01.000Z", "A", "wrld_old:1", "usr_a")],
        vec![
            video("2026-01-01T11:00:01.000Z", "https://v.test/a", "wrld_new:1"),
            video("2026-01-01T11:00:02.000Z", "https://v.test/a", "wrld_new:1"),
        ],
    );

    let sessions = query(&store, GameLogSessionsQueryInput::default());

    assert_eq!(
        sessions
            .iter()
            .map(|s| s.world_id.as_str())
            .collect::<Vec<_>>(),
        vec!["wrld_new", "wrld_old"]
    );
    assert_eq!(sessions[0].events.len(), 1);
    assert_eq!(sessions[0].events[0].type_, "VideoPlay");
    assert_eq!(sessions[0].events[0].play_count, Some(2));
    assert_eq!(sessions[1].events[0].user_id.as_deref(), Some("usr_a"));
}

#[test]
fn returns_every_duration_row_for_the_selected_session_location() {
    let store = test_store("session-player-duration-rows");
    let session_location = "wrld_test:1";
    write_rows(
        &store,
        vec![location(
            "2026-01-01T10:00:00.000Z",
            session_location,
            "wrld_test",
            "Test",
        )],
        vec![
            leave(
                "2025-01-01T10:01:00.000Z",
                "Alice",
                session_location,
                "usr_alice",
                60_000,
            ),
            leave(
                "2026-01-01T10:01:00.000Z",
                "Renamed Alice",
                session_location,
                "usr_alice",
                90_000,
            ),
            leave(
                "2026-01-01T10:02:00.000Z",
                "Ignored by duration calculation",
                session_location,
                "",
                0,
            ),
        ],
        Vec::new(),
    );

    let sessions = query(&store, GameLogSessionsQueryInput::default());

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].player_duration_rows.len(), 3);
    assert_eq!(sessions[0].player_duration_rows[0].display_name, "Alice");
    assert_eq!(sessions[0].player_duration_rows[0].user_id, "usr_alice");
    assert_eq!(sessions[0].player_duration_rows[0].time, 60_000);
    assert_eq!(sessions[0].player_duration_rows[1].time, 90_000);
    assert_eq!(sessions[0].player_duration_rows[2].time, 0);
}

#[test]
fn filters_sessions_by_favorite_user() {
    let store = test_store("sessions-favorite");
    write_rows(
        &store,
        vec![
            location("2026-01-01T10:00:00.000Z", "wrld_a:1", "wrld_a", "A"),
            location("2026-01-01T11:00:00.000Z", "wrld_b:1", "wrld_b", "B"),
        ],
        vec![
            join("2026-01-01T10:00:01.000Z", "A", "wrld_a:1", "usr_a"),
            join("2026-01-01T11:00:01.000Z", "B", "wrld_b:1", "usr_b"),
        ],
        Vec::new(),
    );

    let sessions = query(
        &store,
        GameLogSessionsQueryInput {
            favorite_user_ids: vec!["usr_b".to_string()],
            ..Default::default()
        },
    );

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].world_id, "wrld_b");
    assert_eq!(sessions[0].events[0].is_favorite, Some(true));
}

#[test]
fn global_search_matches_world_name_header() {
    let store = test_store("sessions-search");
    write_rows(
        &store,
        vec![
            location(
                "2026-01-01T10:00:00.000Z",
                "wrld_a:1",
                "wrld_a",
                "Alpha World",
            ),
            location(
                "2026-01-01T11:00:00.000Z",
                "wrld_b:1",
                "wrld_b",
                "Beta World",
            ),
        ],
        vec![
            join("2026-01-01T10:00:01.000Z", "A", "wrld_a:1", "usr_a"),
            join("2026-01-01T11:00:01.000Z", "B", "wrld_b:1", "usr_b"),
        ],
        Vec::new(),
    );

    let sessions = query(
        &store,
        GameLogSessionsQueryInput {
            search: "alpha".to_string(),
            ..Default::default()
        },
    );

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].world_name, "Alpha World");
}

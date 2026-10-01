//! Game-log and player-history commands over the server's synced SQLite:
//! queries, previous-instance lookups and manual entry delete/add, plus
//! the persisted player-list history — ported from the desktop wrappers
//! in `src-tauri/src/commands/local/{game_log,player_list}.rs`.
//!
//! Skipped (desktop-only or unreachable from the server crate):
//! - `app__game_log_persistence_set_disabled` (game_log.rs): gated on the
//!   desktop `GameLogWatcher` host capability and the desktop runtime
//!   host's watcher setter; the server has no live watcher.
//! - `app__game_log_sessions_query`, `app__instance_history_query`
//!   (game_log.rs): input types live in `vrcx_0_application_game`, which
//!   is not a dependency of the server crate and is not re-exported
//!   publicly.
//! - `app__player_list_current_snapshot` (player_list.rs): reads the live
//!   in-game player list from the desktop watcher; the server only holds
//!   persisted game-log data.

use std::sync::Arc;

use serde_json::Value;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_runtime_host_server::local_data::{
    GameLogEntryDeleteKind, GameLogQuery, GameLogWriteKind,
};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

pub fn register(registry: &mut CommandRegistry) {
    // --- game log (local/game_log.rs) ---

    registry.register(
        "app__game_log_entries_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: GameLogWriteKind = arg(&args, "kind")?;
            let entries: Vec<Value> = arg(&args, "entries")?;
            let state = Arc::clone(&state);
            run_blocking("game log entries add", move || {
                state.add_game_log_entries(kind, entries)
            })
            .await?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__game_log_entry_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: GameLogEntryDeleteKind = arg(&args, "kind")?;
            let entry: Value = arg(&args, "entry")?;
            let local_data = state.local_data().clone();
            run_blocking("game log entry delete", move || {
                local_data
                    .game_log_entry_delete(kind, entry)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__game_log_instance_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let location: String = arg(&args, "location")?;
            let event_ids: Vec<i64> = arg(&args, "eventIds")?;
            let local_data = state.local_data().clone();
            run_blocking("game log instance delete", move || {
                local_data
                    .game_log_instance_delete(location, event_ids)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__game_log_instance_delete_by_location",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let location: String = arg(&args, "location")?;
            let local_data = state.local_data().clone();
            run_blocking("game log instance delete", move || {
                local_data
                    .game_log_instance_delete_by_location(location)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__game_log_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: GameLogQuery = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("game log query", move || {
                local_data
                    .game_log_query(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__game_log_previous_instances_by_group_id",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let group_id: String = arg(&args, "groupId")?;
            let local_data = state.local_data().clone();
            run_blocking("game log previous instances query", move || {
                local_data
                    .previous_instances_by_group_id(group_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__game_log_previous_instances_by_world_id",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let world_id: String = arg(&args, "worldId")?;
            let local_data = state.local_data().clone();
            run_blocking("game log previous instances query", move || {
                local_data
                    .previous_instances_by_world_id(world_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- player-list history (local/player_list.rs, persisted subset) ---

    registry.register(
        "app__instance_activity_dates_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("instance activity dates query", move || {
                local_data
                    .instance_activity_dates_get(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__instance_activity_rows_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let start_date: String = arg(&args, "startDate")?;
            let end_date: String = arg(&args, "endDate")?;
            let local_data = state.local_data().clone();
            run_blocking("instance activity rows query", move || {
                local_data
                    .instance_activity_rows_get(start_date, end_date)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__world_summaries_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let world_ids: Vec<String> = arg(&args, "worldIds")?;
            let local_data = state.local_data().clone();
            run_blocking("world summaries query", move || {
                local_data
                    .world_summaries_get(world_ids)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
}

// Commands restored after adding the contracts/application-game deps:
// game-log session/instance history queries and saved group collections.
pub fn register_restored(registry: &mut CommandRegistry) {
    registry.register(
        "app__game_log_sessions_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_application_game::GameLogSessionsQueryInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("game log sessions query", move || {
                local_data
                    .game_log_sessions_query(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__instance_history_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_application_game::InstanceHistoryQueryInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("instance history query", move || {
                local_data
                    .instance_history_query(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__saved_group_collection_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_contracts::SavedGroupCollectionCreateInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("saved group collection create", move || {
                local_data
                    .saved_group_collection_create(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__saved_group_collection_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_contracts::SavedGroupCollectionDeleteInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("saved group collection delete", move || {
                local_data
                    .saved_group_collection_delete(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__saved_group_favorite_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_contracts::SavedGroupFavoriteAddInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("saved group favorite add", move || {
                local_data
                    .saved_group_favorite_add(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__saved_group_favorite_remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_contracts::SavedGroupFavoriteRemoveInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("saved group favorite remove", move || {
                local_data
                    .saved_group_favorite_remove(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__feed_persistence_set_disabled",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let disabled: bool = arg(&args, "disabled")?;
            let local_data = state.local_data().clone();
            run_blocking("feed persistence toggle", move || {
                local_data
                    .set_feed_persistence_disabled(disabled)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await?;
            ok(serde_json::json!(null))
        },
    );
}

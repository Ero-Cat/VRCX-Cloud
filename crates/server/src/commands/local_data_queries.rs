//! Local data query commands: activity views, avatar history/tags,
//! browse history, favorites, mutual graph, notifications, world/file
//! lookups, local moderation and database maintenance — ported from the
//! desktop wrappers in `src-tauri/src/commands/local/` plus the top-level
//! database upgrade commands from `src-tauri/src/commands/database.rs`.
//!
//! Skipped (desktop-only or unreachable from the server crate):
//! - `app__get_vrchat_user_moderation`, `app__set_vrchat_user_moderation`
//!   (local/player_moderations.rs): read/write the VRChat client's
//!   local-files moderation database behind the desktop-only
//!   `VrchatPathDiscovery` host capability.
//! - `app__saved_group_collection_create`, `app__saved_group_collection_delete`,
//!   `app__saved_group_favorite_add`, `app__saved_group_favorite_remove`
//!   (local/favorites.rs): input types live in `vrcx_0_contracts`, which is
//!   not a dependency of the server crate and not re-exported publicly.
//! - `app__database_upgrade_start_fresh` (database.rs): requires the desktop
//!   `DatabaseUpgradeLifecycle` that stops window services and requests an
//!   app restart via the tauri `AppHandle`; the server has no
//!   process-restart machinery.

use std::sync::Arc;

use serde_json::Value;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_application::social::{
    MutualGraphFetchCancelInput, MutualGraphFetchStartInput, MutualGraphFriendRefreshInput,
    UserMutualFriendsListInput,
};
use vrcx_0_application_core::FavoriteEntityKind;
use vrcx_0_runtime_host_server::local_data::{
    ActivityOverlapViewBuildInput, ActivityPageBuildInput, ActivityViewBuildInput, AvatarGetInput,
    AvatarTagInput, AvatarTagsPatchInput, BrowseHistoryEntityKind, BrowseHistoryQueryInput,
    BrowseHistoryRecordInput, NotificationListQueryInput, OwnerId, WorldGetInput,
};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

pub fn register(registry: &mut CommandRegistry) {
    // --- activity (local/activity.rs) ---

    registry.register(
        "app__activity_overlap_view",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ActivityOverlapViewBuildInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("activity overlap view", move || {
                local_data
                    .activity_overlap_view(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__activity_view",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ActivityViewBuildInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("activity view", move || {
                local_data
                    .activity_view(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__activity_page_view",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ActivityPageBuildInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("activity page view", move || {
                local_data
                    .activity_page_view(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- avatars (local/avatars.rs) ---

    registry.register(
        "app__avatar_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarGetInput = arg(&args, "input")?;
            ok(state
                .local_data()
                .avatar_get(input)
                .await
                .map_err(vrcx_0_composition::Error::from)?)
        },
    );
    registry.register(
        "app__avatar_find_by_image_url",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let image_url: String = arg(&args, "imageUrl")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar image url lookup", move || {
                local_data
                    .avatar_find_by_image_url(image_url)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_history_clear",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar history clear", move || {
                local_data
                    .avatar_history_clear(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_history_list",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let limit: i64 = arg(&args, "limit")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar history list", move || {
                local_data
                    .avatar_history_list(user_id, limit)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_usage_ranking",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let limit: i64 = arg(&args, "limit")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar usage ranking", move || {
                local_data
                    .avatar_usage_ranking(user_id, limit)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tag_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let tag: Value = arg(&args, "tag")?;
            let color: Value = arg(&args, "color")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tag add", move || {
                local_data
                    .avatar_tag_add(avatar_id, tag, color)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tag_remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let tag: Value = arg(&args, "tag")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tag remove", move || {
                local_data
                    .avatar_tag_remove(avatar_id, tag)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tag_update_color",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let tag: Value = arg(&args, "tag")?;
            let color: Value = arg(&args, "color")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tag update color", move || {
                local_data
                    .avatar_tag_update_color(avatar_id, tag, color)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_distinct",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let local_data = state.local_data().clone();
            run_blocking("avatar tags distinct", move || {
                local_data
                    .avatar_tags_distinct()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tags get", move || {
                local_data
                    .avatar_tags_get(avatar_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_list",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let local_data = state.local_data().clone();
            run_blocking("avatar tags list", move || {
                local_data
                    .avatar_tags_list()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_patch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let patch: AvatarTagsPatchInput = arg(&args, "patch")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tags patch", move || {
                local_data
                    .avatar_tags_patch(avatar_id, patch)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_remove_all",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tags remove all", move || {
                local_data
                    .avatar_tags_remove_all(avatar_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_tags_replace",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let avatar_id: String = arg(&args, "avatarId")?;
            let entries: Vec<AvatarTagInput> = arg(&args, "entries")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar tags replace", move || {
                local_data
                    .avatar_tags_replace(avatar_id, entries)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_time_spent_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let avatar_id: String = arg(&args, "avatarId")?;
            let time_spent: i64 = arg(&args, "timeSpent")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar time spent add", move || {
                local_data
                    .avatar_time_spent_add(user_id, avatar_id, time_spent)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_time_spent_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let avatar_id: String = arg(&args, "avatarId")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar time spent get", move || {
                local_data
                    .avatar_time_spent_get(user_id, avatar_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_time_spent_list",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("avatar time spent list", move || {
                local_data
                    .avatar_time_spent_list(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- browse history (local/browse_history.rs) ---

    registry.register(
        "app__browse_history_record",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: BrowseHistoryRecordInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("browse history record", move || {
                local_data
                    .browse_history_record(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__browse_history_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: BrowseHistoryQueryInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("browse history query", move || {
                local_data
                    .browse_history_query(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__browse_history_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let owner_user_id: OwnerId = arg(&args, "ownerUserId")?;
            let entity_kind: BrowseHistoryEntityKind = arg(&args, "entityKind")?;
            let entity_id: String = arg(&args, "entityId")?;
            let local_data = state.local_data().clone();
            run_blocking("browse history delete", move || {
                local_data
                    .browse_history_delete(owner_user_id, entity_kind, entity_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__browse_history_clear",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let owner_user_id: OwnerId = arg(&args, "ownerUserId")?;
            let entity_kind: Option<BrowseHistoryEntityKind> = arg(&args, "entityKind")?;
            let local_data = state.local_data().clone();
            run_blocking("browse history clear", move || {
                local_data
                    .browse_history_clear(owner_user_id, entity_kind)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__browse_history_retention_days_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let local_data = state.local_data().clone();
            run_blocking("browse history retention get", move || {
                local_data
                    .browse_history_retention_days_get()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__browse_history_retention_days_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let retention_days: i64 = arg(&args, "retentionDays")?;
            let local_data = state.local_data().clone();
            run_blocking("browse history retention set", move || {
                local_data
                    .browse_history_retention_days_set(retention_days)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- favorites (local/favorites.rs) ---

    registry.register(
        "app__favorite_list",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: FavoriteEntityKind = arg(&args, "kind")?;
            let local_data = state.local_data().clone();
            run_blocking("favorite list", move || {
                local_data
                    .favorite_list(kind)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_local_snapshot",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: FavoriteEntityKind = arg(&args, "kind")?;
            let local_data = state.local_data().clone();
            run_blocking("favorite local snapshot", move || {
                local_data
                    .favorite_snapshot(kind)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_local_world_details_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state
                .local_data()
                .favorite_local_world_details_refresh()
                .await
                .map_err(vrcx_0_composition::Error::from)?)
        },
    );
    registry.register(
        "app__saved_group_favorites_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let local_data = state.local_data().clone();
            run_blocking("saved group favorites get", move || {
                local_data
                    .saved_group_favorites_snapshot()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- mutual graph (local/mutual_graph.rs) ---

    registry.register(
        "app__mutual_graph_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("mutual graph snapshot", move || {
                local_data
                    .mutual_graph_snapshot_get(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__mutual_graph_fetch_cancel",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MutualGraphFetchCancelInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("mutual graph fetch cancel", move || {
                local_data
                    .mutual_graph_fetch_cancel(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__mutual_graph_fetch_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MutualGraphFetchStartInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("mutual graph fetch start", move || {
                local_data
                    .mutual_graph_fetch_start(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__mutual_graph_friend_refresh",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MutualGraphFriendRefreshInput = arg(&args, "input")?;
            ok(state
                .local_data()
                .mutual_graph_friend_refresh(input)
                .await
                .map_err(vrcx_0_composition::Error::from)?)
        },
    );
    registry.register(
        "app__user_mutual_friends_list_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: UserMutualFriendsListInput = arg(&args, "input")?;
            ok(state
                .local_data()
                .user_mutual_friends_list(input)
                .await
                .map_err(vrcx_0_composition::Error::from)?)
        },
    );

    // --- notifications (local/notifications.rs) ---
    // The desktop wrappers also refreshed the tray badge after each
    // mutation; the server has no tray, so only the persistence call
    // remains.

    registry.register(
        "app__notification_add_v1",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let notification: Value = arg(&args, "notification")?;
            let local_data = state.local_data().clone();
            run_blocking("notification add v1", move || {
                local_data
                    .notification_add_v1(user_id, notification)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let id: String = arg(&args, "id")?;
            let local_data = state.local_data().clone();
            run_blocking("notification delete", move || {
                local_data
                    .notification_delete(user_id, id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_expire",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let id: String = arg(&args, "id")?;
            let local_data = state.local_data().clone();
            run_blocking("notification expire", move || {
                local_data
                    .notification_expire(user_id, id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_list_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: NotificationListQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("notification list query", move || {
                local_data
                    .notification_list_query(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_update_expired",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let id: String = arg(&args, "id")?;
            let expired: bool = arg(&args, "expired")?;
            let local_data = state.local_data().clone();
            run_blocking("notification update expired", move || {
                local_data
                    .notification_update_expired(user_id, id, expired)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_v2_expire",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let id: String = arg(&args, "id")?;
            let local_data = state.local_data().clone();
            run_blocking("notification v2 expire", move || {
                local_data
                    .notification_v2_expire(user_id, id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__notification_v2_mark_seen",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let id: String = arg(&args, "id")?;
            let local_data = state.local_data().clone();
            run_blocking("notification v2 mark seen", move || {
                local_data
                    .notification_v2_mark_seen(user_id, id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- worlds and files (local/worlds.rs, local/files.rs) ---

    registry.register(
        "app__world_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldGetInput = arg(&args, "input")?;
            ok(state
                .local_data()
                .world_get(input)
                .await
                .map_err(vrcx_0_composition::Error::from)?)
        },
    );
    registry.register(
        "app__world_friend_visits",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let world_id: String = arg(&args, "worldId")?;
            let local_data = state.local_data().clone();
            run_blocking("world friend visits", move || {
                local_data
                    .world_friend_visits(world_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__file_metadata_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let file_url_or_id: String = arg(&args, "fileUrlOrId")?;
            ok(state.local_data().file_metadata_get(file_url_or_id).await)
        },
    );

    // --- local moderation (local/local_moderation.rs) ---

    registry.register(
        "app__local_moderation_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let owner_user_id: OwnerId = arg(&args, "ownerUserId")?;
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("local moderation get", move || {
                local_data
                    .local_moderation_get(owner_user_id, user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__local_moderation_list",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let owner_user_id: OwnerId = arg(&args, "ownerUserId")?;
            let local_data = state.local_data().clone();
            run_blocking("local moderation list", move || {
                local_data
                    .local_moderation_list(owner_user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- database maintenance (local/database_maintenance.rs) ---

    registry.register(
        "app__database_maintenance_table_sizes_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("maintenance table sizes query", move || {
                local_data
                    .maintenance_table_sizes(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__user_tables_ensure",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("user tables ensure", move || {
                local_data
                    .ensure_user_tables(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );

    // --- database upgrade lifecycle (database.rs, portable subset) ---

    registry.register(
        "app__database_upgrade_preflight",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.database_upgrade().preflight().await?)
        },
    );
    registry.register(
        "app__database_upgrade_run",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.database_upgrade().run().await?)
        },
    );
    registry.register(
        "app__database_upgrade_progress",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.database_upgrade().progress())
        },
    );
    registry.register(
        "app__database_upgrade_retry",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.database_upgrade().retry().await?)
        },
    );
    registry.register(
        "app__database_upgrade_failure_log_path",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.database_upgrade().failure_log_path())
        },
    );
}

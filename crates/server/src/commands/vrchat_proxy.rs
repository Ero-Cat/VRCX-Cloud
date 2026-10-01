//! VRChat remote API proxy commands (worlds / avatars / users / friends /
//! search / tools) — thin wrappers over the `vrchat_remote`, `worlds`,
//! `avatars` and `current_user_mutations` facades.
//!
//! Ported from the desktop Tauri wrappers at commit f9d8539f2
//! (`src-tauri/src/commands/vrchat/{worlds,avatars,users,friends,search,tools}`).
//!
//! Skipped commands: none — every command in the source files maps to a
//! method that still exists on the server facade surface. (The old wrappers
//! in these files contained no blocking SQLite work, so no `run_blocking`.)

use std::sync::Arc;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_application::remote::{
    deserialize_nonnegative_i32, AvatarListSort, AvatarUpdateRequest, CalendarListParams,
    GroupSearchParams, InviteMessageType, QueryOrder, ReleaseStatusFilter, UserSearchParams,
    WorldSearchParams, WorldSearchSort, WorldUpdateRequest,
};
use vrcx_0_application::social::{
    VrchatCurrentUserBadgeInput, VrchatCurrentUserProfileUpdateInput, VrchatCurrentUserTagsInput,
    VrchatCurrentUserUpdateInput,
};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::ok;

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

// ---------------------------------------------------------------------------
// Input payloads (mirror of the old `src-tauri` wrapper types)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldIdInput {
    #[serde(default)]
    world_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldListByUserInput {
    #[serde(default)]
    user_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    offset: i32,
    sort: WorldSearchSort,
    order: QueryOrder,
    release_status: ReleaseStatusFilter,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldSaveInput {
    #[serde(default)]
    world_id: String,
    params: WorldUpdateRequest,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorldPersistentDataInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    world_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvatarIdInput {
    #[serde(default)]
    avatar_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvatarListByUserInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    user: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    offset: i32,
    sort: AvatarListSort,
    order: QueryOrder,
    release_status: ReleaseStatusFilter,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AvatarSaveInput {
    #[serde(default)]
    avatar_id: String,
    params: AvatarUpdateRequest,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserProfileInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    as_self: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserGetInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    force: bool,
    #[serde(default)]
    dialog: bool,
    #[serde(default)]
    is_friend: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FriendUserInput {
    #[serde(default)]
    user_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchWorldsInput {
    #[serde(default)]
    params: WorldSearchParams,
    option: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchUsersInput {
    #[serde(default)]
    params: UserSearchParams,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchGroupsInput {
    #[serde(default)]
    params: GroupSearchParams,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchShortNameInput {
    #[serde(default)]
    short_name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsCalendarGroupInput {
    #[serde(default)]
    group_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsCalendarListInput {
    #[serde(default)]
    params: CalendarListParams,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsCalendarEventInput {
    #[serde(default)]
    group_id: String,
    #[serde(default)]
    event_id: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsFollowGroupEventInput {
    #[serde(default)]
    group_id: String,
    #[serde(default)]
    event_id: String,
    #[serde(default)]
    is_following: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsUserNoteSaveInput {
    #[serde(default)]
    target_user_id: String,
    #[serde(default)]
    note: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsUserReportInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    reason: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsInviteMessagesInput {
    #[serde(default)]
    current_user_id: String,
    message_type: InviteMessageType,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ToolsInviteMessageEditInput {
    #[serde(default)]
    current_user_id: String,
    message_type: InviteMessageType,
    slot: i32,
    #[serde(default)]
    message: String,
}

// ---------------------------------------------------------------------------
// Worlds
// ---------------------------------------------------------------------------

pub fn register(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_world_list_by_user_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldListByUserInput = arg(&args, "input")?;
            ok(state
                .worlds()
                .list_by_user(
                    input.user_id,
                    input.n,
                    input.offset,
                    input.sort,
                    input.order,
                    input.release_status,
                )
                .await?)
        },
    );
    registry.register(
        "app__vrchat_world_persistent_data_exists",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldPersistentDataInput = arg(&args, "input")?;
            ok(state
                .worlds()
                .persistent_data_exists(input.user_id, input.world_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_world_save",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldSaveInput = arg(&args, "input")?;
            ok(state.worlds().save(input.world_id, input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_world_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldIdInput = arg(&args, "input")?;
            ok(state.worlds().delete(input.world_id).await?)
        },
    );
    registry.register(
        "app__vrchat_world_publish",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldIdInput = arg(&args, "input")?;
            ok(state.worlds().publish(input.world_id).await?)
        },
    );
    registry.register(
        "app__vrchat_world_unpublish",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldIdInput = arg(&args, "input")?;
            ok(state.worlds().unpublish(input.world_id).await?)
        },
    );
    registry.register(
        "app__vrchat_world_persistent_data_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: WorldPersistentDataInput = arg(&args, "input")?;
            ok(state
                .worlds()
                .persistent_data_delete(input.user_id, input.world_id)
                .await?)
        },
    );

    // -----------------------------------------------------------------------
    // Avatars
    // -----------------------------------------------------------------------

    registry.register(
        "app__vrchat_avatar_gallery_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .avatar_gallery(input.avatar_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_list_by_user_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarListByUserInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .avatars_by_user(
                    input.user_id,
                    input.user,
                    input.n,
                    input.offset,
                    input.sort,
                    input.order,
                    input.release_status,
                )
                .await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_styles_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.vrchat_remote().avatar_styles().await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_moderations_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.avatars().moderations().await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_select",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().select(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_select_fallback",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().select_fallback(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_save",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarSaveInput = arg(&args, "input")?;
            ok(state.avatars().save(input.avatar_id, input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().delete(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_impostor_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().create_impostor(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_impostor_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().delete_impostor(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_moderation_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().send_moderation(input.avatar_id).await?)
        },
    );
    registry.register(
        "app__vrchat_avatar_moderation_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarIdInput = arg(&args, "input")?;
            ok(state.avatars().delete_moderation(input.avatar_id).await?)
        },
    );

    // -----------------------------------------------------------------------
    // Users
    // -----------------------------------------------------------------------

    registry.register(
        "app__vrchat_user_profile_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: UserProfileInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .user_profile(input.user_id, input.as_self)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_current_user_profile_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatCurrentUserProfileUpdateInput = arg(&args, "input")?;
            ok(state.current_user_mutations().update_profile(input).await?)
        },
    );
    registry.register(
        "app__vrchat_user_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: UserGetInput = arg(&args, "input")?;
            ok(state
                .get_user_via_cache(input.user_id, input.force, input.dialog, input.is_friend)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_user_represented_group_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FriendUserInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .user_represented_group(input.user_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_current_user_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatCurrentUserUpdateInput = arg(&args, "input")?;
            ok(state.current_user_mutations().update_user(input).await?)
        },
    );
    registry.register(
        "app__vrchat_current_user_badge_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatCurrentUserBadgeInput = arg(&args, "input")?;
            ok(state.current_user_mutations().update_badge(input).await?)
        },
    );
    registry.register(
        "app__vrchat_current_user_tags_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatCurrentUserTagsInput = arg(&args, "input")?;
            ok(state.current_user_mutations().add_tags(input).await?)
        },
    );
    registry.register(
        "app__vrchat_current_user_tags_remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatCurrentUserTagsInput = arg(&args, "input")?;
            ok(state.current_user_mutations().remove_tags(input).await?)
        },
    );

    // -----------------------------------------------------------------------
    // Friends
    // -----------------------------------------------------------------------

    registry.register(
        "app__vrchat_friend_status_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FriendUserInput = arg(&args, "input")?;
            ok(state.vrchat_remote().friend_status(input.user_id).await?)
        },
    );

    // -----------------------------------------------------------------------
    // Search
    // -----------------------------------------------------------------------

    registry.register(
        "app__vrchat_search_worlds_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SearchWorldsInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .search_worlds(input.params, input.option)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_search_users_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SearchUsersInput = arg(&args, "input")?;
            ok(state.vrchat_remote().search_users(input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_search_groups_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SearchGroupsInput = arg(&args, "input")?;
            ok(state.vrchat_remote().search_groups(input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_search_groups_strict_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SearchGroupsInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .search_groups_strict(input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_search_instance_short_name_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SearchShortNameInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .search_instance_short_name(input.short_name)
                .await?)
        },
    );

    // -----------------------------------------------------------------------
    // Tools
    // -----------------------------------------------------------------------

    registry.register(
        "app__vrchat_tools_group_calendar_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsCalendarGroupInput = arg(&args, "input")?;
            ok(state.vrchat_remote().group_calendar(input.group_id).await?)
        },
    );
    registry.register(
        "app__vrchat_tools_following_calendars_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsCalendarListInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .following_calendars(input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_group_event_follow",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsFollowGroupEventInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .follow_group_event(input.group_id, input.event_id, input.is_following)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_group_calendar_ics_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsCalendarEventInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .group_calendar_ics(input.group_id, input.event_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_user_note_save",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsUserNoteSaveInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .save_user_note(input.target_user_id, input.note)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_user_report",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsUserReportInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .report_user(input.user_id, input.reason)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_invite_messages_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsInviteMessagesInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .invite_messages(input.current_user_id, input.message_type)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_tools_invite_message_edit",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ToolsInviteMessageEditInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .edit_invite_message(
                    input.current_user_id,
                    input.message_type,
                    input.slot,
                    input.message,
                )
                .await?)
        },
    );
}

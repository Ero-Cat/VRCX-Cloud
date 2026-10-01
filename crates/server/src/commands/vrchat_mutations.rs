//! VRChat mutation commands ported from the desktop bindings: favorites
//! (remote + local), groups, instances, notifications and media.
//!
//! Command names and camelCase argument payloads are identical to the old
//! `src-tauri` wrappers so the existing frontend call surface keeps working
//! over `/api/invoke`.
//!
//! Skipped desktop-only commands:
//! - `app__vrchat_instance_join` — the desktop wrapper drove
//!   `runtime_host().instance_launch()`, which launches the local VRChat
//!   client process; the server has no local game integration and the facade
//!   method was removed.
//!
//! Batch-mutation commands that take a coordinator (e.g.
//! `run_group_membership_batch` / `run_group_moderation_batch`) were defined
//! in the desktop's `application/social_mutation.rs`, not in the five source
//! files ported here; nothing of that shape appears in this batch.

use std::sync::Arc;

use serde::Deserialize;
use vrcx_0_application::favorites::{
    FavoriteRemoteAddInput, FavoriteRemoteDeleteInput, FavoriteRemoteGroupClearInput,
    FavoriteRemoteGroupSaveInput,
};
use vrcx_0_application::media::{
    InventoryItemsCollectInput, LegacyEntityImageKind, LegacyEntityImageUploadInput,
};
use vrcx_0_application::remote::{
    deserialize_nonnegative_i32, EmojiUploadParams, InstanceCreateRequest,
    InventoryItemUpdateRequest, InventoryListParams, MediaAssetUploadRequest, MediaFileListParams,
    PrintUploadParams, ProfileDecorationEquipSlot, RequestInviteRequest,
};
use vrcx_0_application::social::{
    VrchatGroupGalleryInput, VrchatGroupIdInput, VrchatGroupJoinRequestRespondInput,
    VrchatGroupJoinRequestsInput, VrchatGroupLogsInput, VrchatGroupMemberPropsInput,
    VrchatGroupMemberRoleInput, VrchatGroupMembersInput, VrchatGroupMembersSearchInput,
    VrchatGroupPagedInput, VrchatGroupPostCreateInput, VrchatGroupPostDeleteInput,
    VrchatGroupPostEditInput, VrchatGroupProfileInput, VrchatGroupRepresentationInput,
    VrchatGroupUpdateInput, VrchatGroupUserGroupsInput, VrchatGroupUserInput,
};
use vrcx_0_application_core::vrchat_api::require_text;
use vrcx_0_application_core::{FavoriteEntityKind, FavoriteGroupVisibility, VrchatFavoriteType};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

// ---------------------------------------------------------------------------
// Input payloads (mirrors of the desktop `types.rs`, camelCase over the wire)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteWorldsInput {
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    offset: i32,
    #[serde(default)]
    owner_id: String,
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    tag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteGroupsInput {
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    n: i32,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    offset: i32,
    #[serde(default)]
    owner_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteAddInput {
    #[serde(rename = "type")]
    type_name: VrchatFavoriteType,
    #[serde(default)]
    favorite_id: String,
    #[serde(default)]
    tags: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteDeleteInput {
    #[serde(default)]
    object_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteGroupSaveInput {
    #[serde(rename = "type")]
    type_name: VrchatFavoriteType,
    #[serde(default)]
    group: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    visibility: Option<FavoriteGroupVisibility>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatFavoriteGroupClearInput {
    #[serde(rename = "type")]
    type_name: VrchatFavoriteType,
    #[serde(default)]
    group: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalFavoriteInput {
    kind: FavoriteEntityKind,
    #[serde(default)]
    entity_id: String,
    #[serde(default)]
    group_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalFavoriteGroupInput {
    kind: FavoriteEntityKind,
    #[serde(default)]
    group_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalFavoriteGroupRenameInput {
    kind: FavoriteEntityKind,
    #[serde(default)]
    group_name: String,
    #[serde(default)]
    new_group_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatInstanceIdentityInput {
    #[serde(default)]
    world_id: String,
    #[serde(default)]
    instance_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatInstanceShortNameInput {
    #[serde(default)]
    world_id: String,
    #[serde(default)]
    instance_id: String,
    #[serde(default)]
    short_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatInstanceCreateInput {
    params: InstanceCreateRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatInstanceSelfInviteInput {
    #[serde(default)]
    world_id: String,
    #[serde(default)]
    instance_id: String,
    #[serde(default)]
    short_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatInstanceCloseInput {
    #[serde(default)]
    location: String,
    #[serde(default)]
    hard_close: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatRequestInviteSendInput {
    #[serde(default)]
    receiver_user_id: String,
    params: RequestInviteRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatRequestInvitePhotoSendInput {
    #[serde(default)]
    receiver_user_id: String,
    params: RequestInviteRequest,
    #[serde(default)]
    image_data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatBoopInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    emoji_id: String,
    #[serde(default)]
    inventory_item_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaFilesInput {
    #[serde(default)]
    params: MediaFileListParams,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaFileIdInput {
    #[serde(default)]
    file_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaImageUploadInput {
    #[serde(default)]
    image_data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaEmojiUploadInput {
    #[serde(default)]
    image_data: String,
    params: EmojiUploadParams,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaAvatarGalleryImageUploadInput {
    #[serde(default)]
    image_data: String,
    avatar_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaPrintUploadInput {
    #[serde(default)]
    image_data: String,
    #[serde(default)]
    crop_white_border: bool,
    params: PrintUploadParams,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaPrintsInput {
    #[serde(default)]
    user_id: String,
    #[serde(default, deserialize_with = "deserialize_nonnegative_i32")]
    n: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaPrintIdInput {
    #[serde(default)]
    print_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatPrintFavoriteSetInput {
    #[serde(default)]
    print_id: String,
    #[serde(default)]
    favorite: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatPrintFavoritesSetInput {
    #[serde(default)]
    print_ids: Vec<String>,
    #[serde(default)]
    favorite: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaInventoryItemsInput {
    #[serde(default)]
    params: InventoryListParams,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaUserInventoryItemInput {
    #[serde(default)]
    user_id: String,
    #[serde(default)]
    inventory_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaInventoryItemInput {
    #[serde(default)]
    inventory_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaInventoryItemUpdateInput {
    #[serde(default)]
    inventory_id: String,
    params: InventoryItemUpdateRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaInventoryTemplateInput {
    #[serde(default)]
    inventory_template_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaProfileDecorationEquipInput {
    #[serde(default)]
    inventory_id: String,
    equip_slot: ProfileDecorationEquipSlot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaProfileDecorationUnequipInput {
    equip_slot: ProfileDecorationEquipSlot,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaRewardRedeemInput {
    #[serde(default)]
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VrchatMediaLegacyImageUploadInput {
    #[serde(default)]
    entity_id: String,
    #[serde(default)]
    image_url: String,
    #[serde(default)]
    base64_file: String,
    #[serde(default)]
    file_size_in_bytes: Option<i64>,
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

pub fn register(registry: &mut CommandRegistry) {
    register_favorites(registry);
    register_groups(registry);
    register_instances(registry);
    register_notifications(registry);
    register_media(registry);
}

fn register_favorites(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_favorite_worlds_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteWorldsInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .favorite_worlds(
                    input.n,
                    input.offset,
                    input.owner_id,
                    input.user_id,
                    input.tag,
                )
                .await?)
        },
    );
    registry.register(
        "app__vrchat_favorite_groups_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteGroupsInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .favorite_groups(input.n, input.offset, input.owner_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_favorite_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteAddInput = arg(&args, "input")?;
            ok(state
                .favorite_add_remote(FavoriteRemoteAddInput {
                    kind: input.type_name,
                    entity_id: input.favorite_id,
                    tags: input.tags,
                })
                .await?)
        },
    );
    registry.register(
        "app__vrchat_favorite_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteDeleteInput = arg(&args, "input")?;
            ok(state
                .favorite_delete_remote(FavoriteRemoteDeleteInput {
                    object_id: input.object_id,
                })
                .await?)
        },
    );
    registry.register(
        "app__vrchat_favorite_group_save",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteGroupSaveInput = arg(&args, "input")?;
            ok(state
                .favorite_group_save_remote(FavoriteRemoteGroupSaveInput {
                    kind: input.type_name,
                    group: input.group,
                    display_name: input.display_name,
                    visibility: input.visibility,
                })
                .await?)
        },
    );
    registry.register(
        "app__vrchat_favorite_group_clear",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatFavoriteGroupClearInput = arg(&args, "input")?;
            ok(state
                .favorite_group_clear_remote(FavoriteRemoteGroupClearInput {
                    kind: input.type_name,
                    group: input.group,
                })
                .await?)
        },
    );
    registry.register(
        "app__local_favorite_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LocalFavoriteInput = arg(&args, "input")?;
            let entity_id = require_text(input.entity_id, "LocalFavoriteAdd requires entityId.")?;
            let group_name =
                require_text(input.group_name, "LocalFavoriteAdd requires groupName.")?;
            let local_data = state.local_data().clone();
            run_blocking("local favorite add", move || {
                local_data
                    .favorite_add_local(input.kind, entity_id, group_name)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__local_favorite_remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LocalFavoriteInput = arg(&args, "input")?;
            let entity_id =
                require_text(input.entity_id, "LocalFavoriteRemove requires entityId.")?;
            let group_name =
                require_text(input.group_name, "LocalFavoriteRemove requires groupName.")?;
            let local_data = state.local_data().clone();
            run_blocking("local favorite remove", move || {
                local_data
                    .favorite_remove_local(input.kind, entity_id, group_name)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__local_favorite_group_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LocalFavoriteGroupInput = arg(&args, "input")?;
            let group_name = require_text(
                input.group_name,
                "LocalFavoriteGroupCreate requires groupName.",
            )?;
            let favorite_mutations = state
                .runtime()
                .desktop_assembly()
                .favorite_mutations()
                .clone();
            run_blocking("local favorite group create", move || {
                favorite_mutations
                    .create_local_group(input.kind, group_name)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__local_favorite_group_rename",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LocalFavoriteGroupRenameInput = arg(&args, "input")?;
            let group_name = require_text(
                input.group_name,
                "LocalFavoriteGroupRename requires groupName.",
            )?;
            let new_group_name = require_text(
                input.new_group_name,
                "LocalFavoriteGroupRename requires newGroupName.",
            )?;
            let favorite_mutations = state
                .runtime()
                .desktop_assembly()
                .favorite_mutations()
                .clone();
            run_blocking("local favorite group rename", move || {
                favorite_mutations
                    .rename_local_group(input.kind, group_name, new_group_name)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__local_favorite_group_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LocalFavoriteGroupInput = arg(&args, "input")?;
            let group_name = require_text(
                input.group_name,
                "LocalFavoriteGroupDelete requires groupName.",
            )?;
            let favorite_mutations = state
                .runtime()
                .desktop_assembly()
                .favorite_mutations()
                .clone();
            run_blocking("local favorite group delete", move || {
                favorite_mutations
                    .delete_local_group(input.kind, group_name)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
}

/// Every group command takes one typed `input` payload and delegates to the
/// same `ServerGroupRuntime` method the desktop wrapper called.
fn register_groups(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_group_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupProfileInput = arg(&args, "input")?;
            ok(state.groups().get(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_user_groups_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserGroupsInput = arg(&args, "input")?;
            ok(state.groups().user_groups(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_posts_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPagedInput = arg(&args, "input")?;
            ok(state.groups().posts(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_role_ids_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().member_role_ids(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_members_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupMembersInput = arg(&args, "input")?;
            ok(state.groups().members(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_members_search",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupMembersSearchInput = arg(&args, "input")?;
            ok(state.groups().search_members(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_gallery_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupGalleryInput = arg(&args, "input")?;
            ok(state.groups().gallery(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_bans_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPagedInput = arg(&args, "input")?;
            ok(state.groups().bans(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_invites_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPagedInput = arg(&args, "input")?;
            ok(state.groups().invites(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_join_requests_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupJoinRequestsInput = arg(&args, "input")?;
            ok(state.groups().join_requests(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_audit_log_types_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().audit_log_types(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_logs_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupLogsInput = arg(&args, "input")?;
            ok(state.groups().logs(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_user_instances_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserGroupsInput = arg(&args, "input")?;
            ok(state.groups().user_instances(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUpdateInput = arg(&args, "input")?;
            ok(state.groups().update(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().delete(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_post_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPostCreateInput = arg(&args, "input")?;
            ok(state.groups().create_post(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_post_edit",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPostEditInput = arg(&args, "input")?;
            ok(state.groups().edit_post(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_post_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupPostDeleteInput = arg(&args, "input")?;
            ok(state.groups().delete_post(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_join",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().join(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_leave",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().leave(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_request_cancel",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().cancel_request(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_invite_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().send_invite(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_kick",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().kick_member(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_ban",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().ban_member(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_unban",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().unban_member(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_role_add",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupMemberRoleInput = arg(&args, "input")?;
            ok(state.groups().add_member_role(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_role_remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupMemberRoleInput = arg(&args, "input")?;
            ok(state.groups().remove_member_role(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_invite_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().delete_invite(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_join_request_respond",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupJoinRequestRespondInput = arg(&args, "input")?;
            ok(state.groups().respond_join_request(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_representation_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupRepresentationInput = arg(&args, "input")?;
            ok(state.groups().set_representation(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_member_props_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupMemberPropsInput = arg(&args, "input")?;
            ok(state.groups().set_member_props(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_block",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupIdInput = arg(&args, "input")?;
            ok(state.groups().block(input).await?)
        },
    );
    registry.register(
        "app__vrchat_group_unblock",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatGroupUserInput = arg(&args, "input")?;
            ok(state.groups().unblock(input).await?)
        },
    );
}

/// `app__vrchat_instance_join` is desktop-only (it launches the local VRChat
/// client via the instance-launch runtime) and resolves to
/// `unsupportedOnWeb` here.
fn register_instances(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_instance_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatInstanceIdentityInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .instance_get(input.world_id, input.instance_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_instance_short_name_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatInstanceShortNameInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .instance_short_name(input.world_id, input.instance_id, input.short_name)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_instance_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatInstanceCreateInput = arg(&args, "input")?;
            ok(state.vrchat_remote().instance_create(input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_instance_self_invite",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatInstanceSelfInviteInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .instance_self_invite(input.world_id, input.instance_id, input.short_name)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_instance_close",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatInstanceCloseInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .instance_close(input.location, input.hard_close)
                .await?)
        },
    );
}

fn register_notifications(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_request_invite_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatRequestInviteSendInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .request_invite(input.receiver_user_id, input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_request_invite_photo_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatRequestInvitePhotoSendInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .request_invite_photo(input.receiver_user_id, input.params, input.image_data)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_boop_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatBoopInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .boop(input.user_id, input.emoji_id, input.inventory_item_id)
                .await?)
        },
    );
}

fn register_media(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_media_files_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaFilesInput = arg(&args, "input")?;
            ok(state.vrchat_remote().media_files(input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_media_file_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaFileIdInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .delete_media_file(input.file_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_gallery_image_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaImageUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_gallery_image(input.image_data)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_avatar_gallery_image_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaAvatarGalleryImageUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_avatar_gallery_image(input.image_data, input.avatar_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_vrc_plus_icon_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaImageUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_vrc_plus_icon(input.image_data)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_emoji_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaEmojiUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_emoji(input.image_data, input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_sticker_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaImageUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_sticker(input.image_data)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_print_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaPrintUploadInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .upload_print(input.image_data, input.crop_white_border, input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_asset_upload",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MediaAssetUploadRequest = arg(&args, "input")?;
            ok(state.vrchat_remote().upload_media_asset(input).await?)
        },
    );
    registry.register(
        "app__vrchat_media_prints_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaPrintsInput = arg(&args, "input")?;
            ok(state.vrchat_remote().prints(input.user_id, input.n).await?)
        },
    );
    registry.register(
        "app__vrchat_media_print_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaPrintIdInput = arg(&args, "input")?;
            ok(state.vrchat_remote().print(input.print_id).await?)
        },
    );
    registry.register(
        "app__vrchat_media_print_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaPrintIdInput = arg(&args, "input")?;
            ok(state.vrchat_remote().delete_print(input.print_id).await?)
        },
    );
    registry.register(
        "app__vrchat_prints_favorites_list",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.media().print_favorites()?)
        },
    );
    registry.register(
        "app__vrchat_prints_favorite_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatPrintFavoriteSetInput = arg(&args, "input")?;
            ok(state
                .media()
                .set_print_favorite(&input.print_id, input.favorite)?)
        },
    );
    registry.register(
        "app__vrchat_prints_favorites_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatPrintFavoritesSetInput = arg(&args, "input")?;
            ok(state
                .media()
                .set_print_favorites(&input.print_ids, input.favorite)?)
        },
    );
    registry.register(
        "app__vrchat_media_inventory_items_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaInventoryItemsInput = arg(&args, "input")?;
            ok(state.vrchat_remote().inventory_items(input.params).await?)
        },
    );
    registry.register(
        "app__vrchat_media_inventory_items_collect",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: InventoryItemsCollectInput = arg(&args, "input")?;
            ok(state.media().collect_inventory_items(input).await?)
        },
    );
    registry.register(
        "app__vrchat_media_inventory_template_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaInventoryTemplateInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .inventory_template(input.inventory_template_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_profile_decoration_equip",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaProfileDecorationEquipInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .equip_profile_decoration(input.inventory_id, input.equip_slot)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_profile_decoration_unequip",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaProfileDecorationUnequipInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .unequip_profile_decoration(input.equip_slot)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_user_inventory_item_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaUserInventoryItemInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .user_inventory_item(input.user_id, input.inventory_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_inventory_item_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaInventoryItemUpdateInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .update_inventory_item(input.inventory_id, input.params)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_inventory_bundle_consume",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaInventoryItemInput = arg(&args, "input")?;
            ok(state
                .vrchat_remote()
                .consume_inventory_bundle(input.inventory_id)
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_reward_redeem",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaRewardRedeemInput = arg(&args, "input")?;
            ok(state.vrchat_remote().redeem_reward(input.code).await?)
        },
    );
    registry.register(
        "app__vrchat_media_avatar_image_upload_legacy",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaLegacyImageUploadInput = arg(&args, "input")?;
            ok(state
                .media()
                .upload_legacy_entity_image(
                    LegacyEntityImageUploadInput {
                        entity_id: input.entity_id,
                        image_url: input.image_url,
                        base64_file: input.base64_file,
                        file_size_in_bytes: input.file_size_in_bytes,
                    },
                    LegacyEntityImageKind::Avatar,
                    "app__vrchat_media_avatar_image_upload_legacy",
                )
                .await?)
        },
    );
    registry.register(
        "app__vrchat_media_world_image_upload_legacy",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: VrchatMediaLegacyImageUploadInput = arg(&args, "input")?;
            ok(state
                .media()
                .upload_legacy_entity_image(
                    LegacyEntityImageUploadInput {
                        entity_id: input.entity_id,
                        image_url: input.image_url,
                        base64_file: input.base64_file,
                        file_size_in_bytes: input.file_size_in_bytes,
                    },
                    LegacyEntityImageKind::World,
                    "app__vrchat_media_world_image_upload_legacy",
                )
                .await?)
        },
    );
}

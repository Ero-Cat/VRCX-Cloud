//! Social / favorites / notification application commands (friend log
//! resolution, frontend batch mutations, favorite transfer & import, group
//! calendar + quick moderation, notification chains, webhook delivery,
//! presence automation, note export, social baselines, quick search, share
//! collections, realtime friend loads and my-avatars queries).
//!
//! Ported from the desktop `src-tauri/src/commands/application/*` wrappers;
//! command names and camelCase argument keys are identical so the shared
//! frontend call surface works unchanged over HTTP.
//!
//! Desktop-only pieces intentionally skipped here (no facade method on
//! `ServerRuntimeHostState`, or player-machine shell surface):
//! - `refresh_tray_notification` after notification-chain actions
//!   (desktop tray refresh; the web panel relies on events instead).
//! - `app__friend_log_current_list` / `app__friend_log_history_query`
//!   (already registered in `local.rs`).
//! - background_image / background_mode / community_theme / deep_link /
//!   desktop_notification / vr_overlay / registry_backup (desktop shell).

use std::sync::{Arc, OnceLock};

use serde_json::{json, Value};
use vrcx_0_application::avatars::{MyAvatarByIdInput, MyAvatarsInput};
use vrcx_0_application::collections::{
    ShareCollectionCreateInput, SharedCollectionImportStartInput,
};
use vrcx_0_application::favorites::{
    FavoriteBulkRemoveInput, FavoriteCacheSnapshotInput, FavoriteDetailsHydrateInput,
    FavoriteDetailsRuntime, FavoriteImportStartInput, FavoriteTransferSelectionInput,
};
use vrcx_0_application::social::{
    AvatarContentTagsBatchInput, FriendLogNameResolutionCoordinator, FriendLogNameResolutionInput,
    GroupBanImportStartInput, GroupCalendarInput, GroupMembershipBatchCoordinator,
    GroupMembershipBatchInput, GroupModerationBatchCoordinator, GroupModerationBatchInput,
    GroupQuickModerationActionInput, GroupQuickModerationInput, InstanceInviteBatchInput,
    ModerationSyncMutationInput, ModerationSyncRefreshInput, NoteExportStartInput,
    NotificationBoopDismissInput, NotificationBoopReplyInput, NotificationHideExpireInput,
    NotificationInstanceInviteInput, NotificationInviteResponseInput,
    NotificationMarkSeenBatchInput, NotificationRequestInviteAcceptInput, NotificationRespondInput,
    QuickSearchQueryInput, QuickSearchRuntime, SocialFriendMutationInput,
    SocialFriendRequestAcceptInput, SocialFriendRequestCancelInput, SocialUnfriendBatchInput,
    UserDialogTabCountsInput, UserDialogTabCountsRuntime, UserGroupsOverviewInput,
};
use vrcx_0_application_activity::notification::{
    filter_generic_webhook_payload, parse_webhook_fields, webhook_local_time_string,
    NotificationWebhookFormat,
};
use vrcx_0_application_game::PresenceAutomationRuleKind;
use vrcx_0_application_realtime::{SocialFavoritesBaselineInput, SocialFriendRosterBaselineInput};
use vrcx_0_core::json::RawJson;
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

// The desktop kept these coordinators/runtimes alive on `AppState` so that
// cancellation and caching work across commands; the server state facade
// does not expose storage for them, so they live here as process-lifetime
// singletons (the self-hosted server serves a single profile at a time).
static FRIEND_LOG_RESOLUTIONS: OnceLock<FriendLogNameResolutionCoordinator> = OnceLock::new();
static GROUP_MEMBERSHIP_BATCHES: OnceLock<GroupMembershipBatchCoordinator> = OnceLock::new();
static GROUP_MODERATION_BATCHES: OnceLock<GroupModerationBatchCoordinator> = OnceLock::new();
static USER_DIALOG_TAB_COUNTS: OnceLock<UserDialogTabCountsRuntime> = OnceLock::new();
static QUICK_SEARCH: OnceLock<QuickSearchRuntime> = OnceLock::new();
static FAVORITE_DETAILS: OnceLock<FavoriteDetailsRuntime> = OnceLock::new();

fn webhook_test_payload(format: NotificationWebhookFormat, fields: &str) -> Value {
    let timestamp = chrono::Utc::now().to_rfc3339();
    if format == NotificationWebhookFormat::Discord {
        json!({
            "content": null,
            "embeds": [{
                "title": "VRCX-0 webhook test",
                "description": "Webhook delivery is configured.",
                "timestamp": &timestamp,
            }]
        })
    } else {
        let payload = json!({
            "version": 1,
            "event": "test",
            "category": "systemSafety",
            "title": "VRCX-0 webhook test",
            "message": "Webhook delivery is configured.",
            "user": {
                "id": "",
                "displayName": "VRCX-0",
            },
            "location": "VRCX-0 test world public",
            "locationId": "wrld_00000000-0000-0000-0000-000000000000:12345",
            "worldId": "wrld_00000000-0000-0000-0000-000000000000",
            "worldName": "VRCX-0 test world",
            "timestamp": &timestamp,
            "localTime": webhook_local_time_string(&timestamp),
        });
        filter_generic_webhook_payload(payload, &parse_webhook_fields(fields))
    }
}

pub fn register(registry: &mut CommandRegistry) {
    // ----- friend_log.rs -----
    registry.register(
        "app__friend_log_names_resolve",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FriendLogNameResolutionInput = arg(&args, "input")?;
            let coordinator = FRIEND_LOG_RESOLUTIONS.get_or_init(Default::default);
            ok(state.resolve_friend_log_names(coordinator, input).await?)
        },
    );
    registry.register(
        "app__friend_log_names_cancel",
        |_state: Arc<ServerRuntimeHostState>, args| async move {
            let request_id: String = arg(&args, "requestId")?;
            let coordinator = FRIEND_LOG_RESOLUTIONS.get_or_init(Default::default);
            ok(coordinator.cancel(&request_id))
        },
    );

    // ----- frontend_batch.rs -----
    registry.register(
        "app__favorite_import_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FavoriteImportStartInput = arg(&args, "input")?;
            run_blocking("favorite import start", move || {
                state.favorite_import_start(input)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_import_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("favorite import status", move || {
                Ok(state.favorite_import_status())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_import_cancel",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("favorite import cancel", move || {
                Ok(state.favorite_import_cancel())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_import_dismiss",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let run_id: String = arg(&args, "runId")?;
            run_blocking("favorite import dismiss", move || {
                Ok(state.favorite_import_dismiss(&run_id))
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__group_ban_import_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupBanImportStartInput = arg(&args, "input")?;
            run_blocking("group ban import start", move || {
                state.group_ban_import_start(input)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__group_ban_import_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("group ban import status", move || {
                Ok(state.group_ban_import_status())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__group_ban_import_cancel",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("group ban import cancel", move || {
                Ok(state.group_ban_import_cancel())
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__favorite_details_hydrate",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FavoriteDetailsHydrateInput = arg(&args, "input")?;
            let expected_scope = state.require_active_scope("Batch action")?;
            let runtime = FAVORITE_DETAILS.get_or_init(|| state.favorite_details_runtime());
            ok(runtime.hydrate(input, expected_scope).await?)
        },
    );
    registry.register(
        "app__favorite_cache_snapshot",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FavoriteCacheSnapshotInput = arg(&args, "input")?;
            // The desktop additionally refreshed the world card in the
            // favorite-details cache; on the server the hydrate command
            // re-populates it on demand.
            run_blocking("favorite cache snapshot", move || {
                state.persist_favorite_cache_snapshot(input)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__avatar_content_tags_batch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AvatarContentTagsBatchInput = arg(&args, "input")?;
            ok(state.run_avatar_content_tags_batch(input).await?)
        },
    );
    registry.register(
        "app__group_membership_batch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupMembershipBatchInput = arg(&args, "input")?;
            let coordinator = GROUP_MEMBERSHIP_BATCHES.get_or_init(Default::default);
            ok(state.run_group_membership_batch(coordinator, input).await?)
        },
    );
    registry.register(
        "app__group_moderation_batch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupModerationBatchInput = arg(&args, "input")?;
            let coordinator = GROUP_MODERATION_BATCHES.get_or_init(Default::default);
            ok(state.run_group_moderation_batch(coordinator, input).await?)
        },
    );
    registry.register(
        "app__notification_mark_seen_batch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationMarkSeenBatchInput = arg(&args, "input")?;
            ok(state.mark_notifications_seen_batch(input).await?)
        },
    );
    registry.register(
        "app__instance_invite_batch",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: InstanceInviteBatchInput = arg(&args, "input")?;
            ok(state.send_instance_invites_batch(input).await?)
        },
    );
    registry.register(
        "app__notification_sync",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.sync_notifications().await?)
        },
    );

    // ----- favorite_transfer.rs -----
    registry.register(
        "app__favorites_transfer_selection",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FavoriteTransferSelectionInput = arg(&args, "input")?;
            ok(state.transfer_favorite_selection(input).await?)
        },
    );
    registry.register(
        "app__favorites_remove_selection",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: FavoriteBulkRemoveInput = arg(&args, "input")?;
            ok(state.remove_favorite_selection(input).await?)
        },
    );

    // ----- group_calendar.rs -----
    registry.register(
        "app__group_calendar_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupCalendarInput = arg(&args, "input")?;
            ok(state.groups().calendar(input).await?)
        },
    );

    // ----- group_quick_moderation.rs -----
    registry.register(
        "app__user_group_quick_moderation_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupQuickModerationInput = arg(&args, "input")?;
            ok(state.groups().quick_moderation(input).await?)
        },
    );
    registry.register(
        "app__user_group_quick_moderation_action",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: GroupQuickModerationActionInput = arg(&args, "input")?;
            ok(state.groups().run_quick_moderation_action(input).await?)
        },
    );

    // ----- user_groups_overview.rs -----
    registry.register(
        "app__user_groups_overview_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: UserGroupsOverviewInput = arg(&args, "input")?;
            ok(state.groups().user_groups_overview(input).await?)
        },
    );

    // ----- notification_chains.rs (tray refresh is desktop-only) -----
    registry.register(
        "app__notification_hide_and_expire",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationHideExpireInput = arg(&args, "input")?;
            ok(state.social().hide_and_expire_notification(input).await?)
        },
    );
    registry.register(
        "app__notification_request_invite_accept",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationRequestInviteAcceptInput = arg(&args, "input")?;
            ok(state
                .social()
                .accept_request_invite_notification(input)
                .await?)
        },
    );
    registry.register(
        "app__notification_instance_invite_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationInstanceInviteInput = arg(&args, "input")?;
            ok(state
                .social()
                .send_instance_invite_notification(input)
                .await?)
        },
    );
    registry.register(
        "app__notification_invite_response_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationInviteResponseInput = arg(&args, "input")?;
            ok(state
                .social()
                .send_invite_response_notification(input)
                .await?)
        },
    );
    registry.register(
        "app__notification_boop_dismiss",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationBoopDismissInput = arg(&args, "input")?;
            ok(state.social().dismiss_boop_notifications(input).await?)
        },
    );
    registry.register(
        "app__notification_boop_reply",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationBoopReplyInput = arg(&args, "input")?;
            ok(state.social().send_boop_reply_notification(input).await?)
        },
    );
    registry.register(
        "app__notification_respond_and_expire",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NotificationRespondInput = arg(&args, "input")?;
            ok(state
                .social()
                .respond_and_expire_notification(input)
                .await?)
        },
    );

    // ----- webhook delivery (desktop: host/overlay_notifications.rs) -----
    registry.register(
        "app__webhook_send_test",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let url: String = arg(&args, "url")?;
            let format: NotificationWebhookFormat = arg(&args, "format")?;
            let fields: String = arg(&args, "fields")?;
            let url = url.trim();
            if url.is_empty() {
                return Err(ApiError::Message("Webhook URL is required.".into()));
            }
            let payload = webhook_test_payload(format, &fields);
            ok(state
                .send_test_webhook(url.to_string(), format, payload)
                .await?)
        },
    );
    registry.register(
        "app__webhook_delivery_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.webhook_delivery_snapshot())
        },
    );

    // ----- presence_automation.rs -----
    registry.register(
        "app__presence_automation_rules_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: PresenceAutomationRuleKind = arg(&args, "kind")?;
            run_blocking("presence automation rules get", move || {
                state.presence_automation_rules(kind)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__presence_automation_rules_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: PresenceAutomationRuleKind = arg(&args, "kind")?;
            let rules: Vec<RawJson> = arg(&args, "rules")?;
            run_blocking("presence automation rules set", move || {
                state.set_presence_automation_rules(kind, rules)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__presence_automation_rule_enabled_set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let kind: PresenceAutomationRuleKind = arg(&args, "kind")?;
            let rule_id: String = arg(&args, "ruleId")?;
            let enabled: bool = arg(&args, "enabled")?;
            run_blocking("presence automation rule enabled set", move || {
                state.set_presence_automation_rule_enabled(kind, &rule_id, enabled)
            })
            .await
            .and_then(ok)
        },
    );

    // ----- note_export.rs -----
    registry.register(
        "app__note_export_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: NoteExportStartInput = arg(&args, "input")?;
            run_blocking("note export start", move || state.start_note_export(input))
                .await
                .and_then(ok)
        },
    );
    registry.register(
        "app__note_export_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("note export status", move || Ok(state.note_export_status()))
                .await
                .and_then(ok)
        },
    );
    registry.register(
        "app__note_export_cancel",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("note export cancel", move || Ok(state.cancel_note_export()))
                .await
                .and_then(ok)
        },
    );

    // ----- social_mutation.rs -----
    registry.register(
        "app__social_unfriend",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFriendMutationInput = arg(&args, "input")?;
            ok(state.social().unfriend(input).await?)
        },
    );
    registry.register(
        "app__social_unfriend_selection",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialUnfriendBatchInput = arg(&args, "input")?;
            ok(state.social().unfriend_selection(input).await?)
        },
    );
    registry.register(
        "app__social_friend_request_send",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFriendMutationInput = arg(&args, "input")?;
            ok(state.social().send_friend_request(input).await?)
        },
    );
    registry.register(
        "app__social_friend_request_cancel",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFriendRequestCancelInput = arg(&args, "input")?;
            ok(state.social().cancel_friend_request(input).await?)
        },
    );
    registry.register(
        "app__social_friend_request_notification_accept",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFriendRequestAcceptInput = arg(&args, "input")?;
            ok(state
                .social()
                .accept_friend_request_notification(input)
                .await?)
        },
    );

    // ----- social_baseline -----
    registry.register(
        "app__social_baseline_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.refresh_social_baseline().await?)
        },
    );
    registry.register(
        "app__social_favorites_baseline_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFavoritesBaselineInput = arg(&args, "input")?;
            ok(state.social().favorites_baseline(input).await?)
        },
    );
    registry.register(
        "app__social_friend_roster_baseline_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SocialFriendRosterBaselineInput = arg(&args, "input")?;
            ok(state.social().friend_roster_baseline(input).await?)
        },
    );

    // ----- moderation_sync.rs -----
    registry.register(
        "app__moderation_sync_refresh",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ModerationSyncRefreshInput = arg(&args, "input")?;
            ok(state.social().moderation_refresh(input).await?)
        },
    );
    registry.register(
        "app__moderation_sync_update",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ModerationSyncMutationInput = arg(&args, "input")?;
            ok(state.social().moderation_update(input).await?)
        },
    );

    // ----- user_dialog_tab_counts.rs -----
    registry.register(
        "app__user_dialog_tab_counts_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: UserDialogTabCountsInput = arg(&args, "input")?;
            let runtime = USER_DIALOG_TAB_COUNTS.get_or_init(UserDialogTabCountsRuntime::new);
            ok(state.user_dialog_tab_counts(runtime, input).await?)
        },
    );

    // ----- quick_search.rs -----
    registry.register(
        "app__quick_search_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: QuickSearchQueryInput = arg(&args, "input")?;
            let runtime = QUICK_SEARCH.get_or_init(|| state.quick_search_runtime());
            ok(runtime.query(input, state.friend_snapshot()).await?)
        },
    );
    registry.register(
        "app__quick_search_working_set_invalidate",
        |_state: Arc<ServerRuntimeHostState>, _args| async move {
            if let Some(runtime) = QUICK_SEARCH.get() {
                runtime.invalidate_remote_working_set();
            }
            ok(json!(null))
        },
    );

    // ----- share_collection.rs -----
    registry.register(
        "app__share_collection_create",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: ShareCollectionCreateInput = arg(&args, "input")?;
            ok(state.share_collection_create(input).await?)
        },
    );
    registry.register(
        "app__share_collection_open_manage",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            // The desktop opened the browser itself; the web frontend opens
            // the manager URL it gets back here in a new tab.
            ok(json!(state.shared_collection_manager_url().await?))
        },
    );
    registry.register(
        "app__share_collection_preview",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let id: String = arg(&args, "id")?;
            ok(state.preview_shared_collection(&id).await?)
        },
    );
    registry.register(
        "app__world_open_register",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let world_id: String = arg(&args, "worldId")?;
            state.register_world_open_share(world_id).await;
            ok(json!(null))
        },
    );
    registry.register(
        "app__shared_collection_import_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SharedCollectionImportStartInput = arg(&args, "input")?;
            run_blocking("shared collection import start", move || {
                state.start_shared_collection_import(input)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__shared_collection_import_status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("shared collection import status", move || {
                Ok(state.shared_collection_import_status())
            })
            .await
            .and_then(ok)
        },
    );

    // ----- realtime.rs -----
    registry.register(
        "app__current_user_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.refresh_current_user().await?)
        },
    );
    registry.register(
        "app__ingest_user_facts",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let entries: Vec<Value> = arg(&args, "entries")?;
            state.ingest_user_facts(entries);
            ok(json!(null))
        },
    );
    registry.register(
        "app__friend_profile_load_start",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.start_friend_profile_bulk_load()?)
        },
    );
    registry.register(
        "app__friend_profile_load_cancel",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.cancel_friend_profile_bulk_load()?)
        },
    );

    // ----- my_avatars.rs -----
    registry.register(
        "app__my_avatars_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MyAvatarsInput = arg(&args, "input")?;
            ok(state.avatars().my_avatars(input).await?)
        },
    );
    registry.register(
        "app__my_avatar_by_id_get",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: MyAvatarByIdInput = arg(&args, "input")?;
            ok(state.avatars().my_avatar_by_id(input).await?)
        },
    );
}

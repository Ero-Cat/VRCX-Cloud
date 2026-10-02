//! Ancillary runtime snapshot for web hydration.
//!
//! Desktop-only surfaces (community themes, background image, app
//! updater, game client debug state, live game process) report neutral
//! defaults; server-backed surfaces carry live status. Most importantly
//! the privacy-lock state hydrates with the authenticated user id so the
//! frontend does not sit on the "pending" lock overlay forever waiting
//! for an event that fired before its WebSocket connected.

use serde::Serialize;
use vrcx_0_application::profile::{
    AppUpdateDownloadStatusSnapshot, AppUpdateStatusSnapshot, BackgroundImageProjection,
    DataDirMigrationStatus, ProfileBackupStatus,
};
use vrcx_0_application::social::MutualGraphFetchStatus;
use vrcx_0_application_game::NowPlayingSnapshot;

use crate::privacy_lock::PrivacyLockSnapshot;
use crate::state::ServerRuntimeHostState;

/// Mirrors the desktop `NotificationDoNotDisturbSnapshot` contract
/// (`{revision, mode, endsAt}`); the server has no desktop DND runtime,
/// so it always reports off.
#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ServerNotificationDoNotDisturbSnapshot {
    pub revision: u64,
    pub mode: &'static str,
    pub ends_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AncillaryRuntimeSnapshot {
    pub community_theme_state: Option<serde_json::Value>,
    pub profile_backup_current_status: ProfileBackupStatus,
    pub data_dir_migration_current_status: DataDirMigrationStatus,
    pub mutual_graph_fetch_status: MutualGraphFetchStatus,
    pub app_update_status: AppUpdateStatusSnapshot,
    pub app_update_download_status: AppUpdateDownloadStatusSnapshot,
    pub game_client_debug_logging_status: Option<serde_json::Value>,
    pub game_process_snapshot: Option<serde_json::Value>,
    pub now_playing: NowPlayingSnapshot,
    pub background_image_state: BackgroundImageProjection,
    pub notification_do_not_disturb_state: ServerNotificationDoNotDisturbSnapshot,
    pub privacy_lock_state: PrivacyLockSnapshot,
}

fn idle_app_update_status() -> AppUpdateStatusSnapshot {
    AppUpdateStatusSnapshot {
        has_available_update: false,
        checked_at: String::new(),
        detail: "Updates are managed by the server deployment.".into(),
        error: None,
        release: None,
        should_notify: false,
    }
}

fn idle_app_update_download_status() -> AppUpdateDownloadStatusSnapshot {
    AppUpdateDownloadStatusSnapshot {
        phase: vrcx_0_application::profile::AppUpdateDownloadPhase::Idle,
        version: None,
        started_at: None,
        downloaded_bytes: 0,
        total_bytes: 0,
        percent: 0,
        error: None,
    }
}

fn disabled_background_image_projection() -> BackgroundImageProjection {
    BackgroundImageProjection {
        revision: 0,
        enabled: false,
        mode: vrcx_0_application::profile::BackgroundImageMode::Off,
        provider_id: vrcx_0_application::profile::BackgroundImageProviderId::NasaEpic,
        custom_source: None,
        snapshot: None,
        error: None,
    }
}

pub async fn ancillary_runtime_snapshot(
    state: &ServerRuntimeHostState,
) -> AncillaryRuntimeSnapshot {
    AncillaryRuntimeSnapshot {
        community_theme_state: None,
        profile_backup_current_status: state.profile_backup_status(),
        data_dir_migration_current_status: state.data_dir_migration_status(),
        mutual_graph_fetch_status: state.mutual_graph_fetch_status(),
        app_update_status: idle_app_update_status(),
        app_update_download_status: idle_app_update_download_status(),
        game_client_debug_logging_status: None,
        game_process_snapshot: None,
        now_playing: NowPlayingSnapshot::default(),
        background_image_state: disabled_background_image_projection(),
        notification_do_not_disturb_state: ServerNotificationDoNotDisturbSnapshot {
            revision: 0,
            mode: "off",
            ends_at: None,
        },
        privacy_lock_state: state.privacy_lock().snapshot(),
    }
}

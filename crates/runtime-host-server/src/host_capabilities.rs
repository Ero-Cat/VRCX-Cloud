//! Host capability report for the server runtime.
//!
//! Shape-compatible with the desktop `HostCapabilities` contract the
//! frontend already consumes (same field names/types in the generated
//! bindings); every capability that requires access to the player's
//! machine is reported as unsupported so the web UI degrades gracefully.

use serde::Serialize;

pub use vrcx_0_platform::host_capabilities::{
    current_host_architecture, current_host_platform, HostArchitecture, HostPlatform,
};

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityStatus {
    pub supported: bool,
    pub enabled: bool,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum LinuxPackageKind {
    #[default]
    Unknown,
    Appimage,
    Deb,
    Rpm,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HostCapabilities {
    pub platform: HostPlatform,
    pub arch: HostArchitecture,
    pub linux_package_kind: LinuxPackageKind,
    pub local_database: CapabilityStatus,
    pub websocket_runtime: CapabilityStatus,
    pub game_log_watcher: CapabilityStatus,
    pub runtime_game_log_ingest: CapabilityStatus,
    pub runtime_game_log_side_effects: CapabilityStatus,
    pub runtime_game_client_lifecycle: CapabilityStatus,
    pub runtime_realtime_transport: CapabilityStatus,
    pub game_process_monitor: CapabilityStatus,
    pub vrchat_path_discovery: CapabilityStatus,
    pub steam_library_discovery: CapabilityStatus,
    pub steam_runtime_integration: CapabilityStatus,
    pub registry_prefs: CapabilityStatus,
    pub game_launch: CapabilityStatus,
    pub vrchat_launch_pipe: CapabilityStatus,
    pub screenshot_cache: CapabilityStatus,
}

impl CapabilityStatus {
    fn available() -> Self {
        Self {
            supported: true,
            enabled: true,
            available: true,
            reason: None,
        }
    }

    fn unsupported(label: &str) -> Self {
        Self {
            supported: false,
            enabled: false,
            available: false,
            reason: Some(format!(
                "{label} requires a desktop device; it is not available on the server"
            )),
        }
    }
}

/// Capability set for the web server runtime: local database and the
/// realtime transport are live; everything driven by the player's machine
/// (game logs, process state, screenshots, launching) is unsupported —
/// that data arrives via remote sync from the user's desktop instead.
pub fn server_host_capabilities() -> HostCapabilities {
    let unsupported_labels = [
        "GameLog watcher",
        "runtime GameLog ingest",
        "runtime GameLog side effects",
        "runtime game client lifecycle",
        "Game process monitor",
        "VRChat path discovery",
        "Steam library discovery",
        "Steam runtime integration",
        "VRChat registry preferences",
        "Game launch",
        "VRChat launch pipe",
        "Screenshot cache",
    ];
    let unsupported = |index: usize| CapabilityStatus::unsupported(unsupported_labels[index]);
    let available = CapabilityStatus::available();
    HostCapabilities {
        platform: current_host_platform(),
        arch: current_host_architecture(),
        linux_package_kind: LinuxPackageKind::Unknown,
        local_database: available.clone(),
        websocket_runtime: available.clone(),
        game_log_watcher: unsupported(0),
        runtime_game_log_ingest: unsupported(1),
        runtime_game_log_side_effects: unsupported(2),
        runtime_game_client_lifecycle: unsupported(3),
        runtime_realtime_transport: available,
        game_process_monitor: unsupported(4),
        vrchat_path_discovery: unsupported(5),
        steam_library_discovery: unsupported(6),
        steam_runtime_integration: unsupported(7),
        registry_prefs: unsupported(8),
        game_launch: unsupported(9),
        vrchat_launch_pipe: unsupported(10),
        screenshot_cache: unsupported(11),
    }
}

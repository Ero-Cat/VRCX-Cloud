//! Host-neutral runtime facade for the self-hosted web server.
//!
//! Wires the composition `RuntimeHostState` (auth, realtime transport,
//! persistence, remote sync) to the server-safe query/mutation runtimes the
//! web command layer calls into. Game-log ingest, VR overlay, tray/window
//! shell and other player-machine capabilities intentionally do not exist
//! here; that data arrives via the remote sync protocol instead.

mod ancillary_snapshot;
mod assistant_adapters;
pub mod auth_failure;
mod avatar;
mod background_image;
mod background_remote;
mod context;
mod current_user_mutation;
pub mod data_dir;
mod database_upgrade;
mod external_api;
mod game_state_store;
mod group;
mod host_actions;
mod host_capabilities;
mod host_file_access;
pub mod local_data;
mod mcp_adapters;
mod media;
mod privacy_lock;
mod profile_backup;
mod profile_bio;
mod proxy_connectivity;
mod social;
mod startup_bootstrap;
mod state;
mod vrchat_api;
mod vrchat_remote;
mod world_remote;

pub use background_image::{
    background_image_files_from_paths, HostBackgroundImageFileResolver, BACKGROUND_IMAGE_EXTENSIONS,
};
pub use context::ServerRuntimeServices;
pub use database_upgrade::{DatabaseUpgradeLifecycle, ServerDatabaseUpgradeRuntime};
pub use external_api::{ExternalApiExecuteResponse, ExternalApiRuntime};
pub use group::ServerGroupRuntime;
pub use host_actions::{RuntimeHost, RuntimeHostActions};
pub use host_capabilities::{
    server_host_capabilities, CapabilityStatus, HostCapabilities, LinuxPackageKind,
};
pub use host_file_access::{is_known_root_path, HostFileAccess};
pub use media::ServerMediaRuntime;
pub use privacy_lock::{PrivacyLockOutcome, PrivacyLockRuntime, PrivacyLockSnapshot};
pub use profile_backup::{ServerProfileBackupRuntime, ServerProfileRestoreRequest};
pub use proxy_connectivity::test_proxy_connectivity;
pub use social::ServerSocialRuntime;
pub use startup_bootstrap::{system_culture, system_language, StartupBootstrapSnapshot};
pub use state::{
    CurrentUserRefreshOutcome, RuntimeJobRecordInput, ServerRuntimeHostOptions,
    ServerRuntimeHostState,
};
pub use vrchat_remote::ServerVrchatRemoteFacade;
pub use vrcx_0_composition::{Error, Result};

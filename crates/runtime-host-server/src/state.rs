//! Server runtime host state.
//!
//! Wraps the composition [`RuntimeHostState`] with the server-safe
//! query/mutation runtimes the web command layer drives. Game-log ingest,
//! the VR overlay, tray/window shell integration and the app updater are
//! desktop-only and intentionally absent; game-log *data* arrives via the
//! remote sync protocol and is queried through the same persistence layer.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use vrcx_0_application_core::RuntimeOperationStatus;

use crate::avatar::ServerAvatarRuntime;
use crate::context::ServerRuntimeServicesDeps;
use crate::data_dir::ServerDataDirRuntime;
use crate::external_api::ExternalApiRuntime;
use crate::group::ServerGroupRuntime;
use crate::host_file_access::HostFileAccess;
use crate::local_data::LocalDataRuntime;
use crate::media::ServerMediaRuntime;
use crate::privacy_lock::PrivacyLockRuntime;
use crate::profile_backup::ServerProfileBackupRuntime;
use crate::social::ServerSocialRuntime;
use crate::vrchat_remote::ServerVrchatRemoteFacade;
use serde_json::Value;
use vrcx_0_application::auth::{
    AutoLoginOutcome, AutoLoginStartInput, LoginSessionCancelInput, LoginSessionEnd,
    LoginSessionRespondInput, LoginSessionStartInput, LoginSessionState, SavedAuthSnapshot,
    VrchatConfigRuntime,
};
use vrcx_0_application::collections::{
    get_or_create_share_owner_token, register_world_open_share, share_collection_create,
    ShareCollectionCreateInput, ShareCollectionCreateResult, ShareCollectionDeps,
    SharedCollectionImportStartInput, SharedCollectionImportStatus,
};
use vrcx_0_application::favorites::{
    FavoriteBulkRemoveInput, FavoriteBulkRemoveResult, FavoriteCacheSnapshotInput,
    FavoriteImportStartInput, FavoriteImportStatus, FavoriteTransferSelectionInput,
    FavoriteTransferSelectionResult,
};
use vrcx_0_application::profile::DatabaseUpgradeRuntime;
use vrcx_0_application::remote::WorldRemoteRuntime;
use vrcx_0_application::social::{
    CurrentUserMutationRuntime, GroupBanImportStartInput, GroupBanImportStatus,
};
use vrcx_0_application_core::UnavailableLocalGameContextSource;
use vrcx_0_application_core::{
    BackendRuntimePhase, BackendRuntimeTelemetryKind, FriendProfileLoadStatusPayload,
    RuntimeEventSink, RuntimeTaskExecutor, TaskStopToken, VrcStatusSnapshot,
};
use vrcx_0_application_game::PresenceAutomationRuleKind;
use vrcx_0_composition::{
    BackendRuntimeCombinedSnapshot, Result, RuntimeHostComposition, RuntimeHostOptions,
    RuntimeHostProfile, RuntimeHostProfileExtension, RuntimeHostServerAssemblyDeps,
    RuntimeHostState, RuntimeHostStateBuilder, UnavailableGroupOrderSource,
};
use vrcx_0_core::json::RawJson;
use vrcx_0_platform::app_paths::AppDataDirResolution;

mod background_ticks;

use background_ticks::{
    run_background_presence_tick, BackgroundTickContext, BACKGROUND_PRESENCE_AUTOMATION_JOB,
    BACKGROUND_PRESENCE_CADENCE_SECONDS,
};

const USER_GENERATED_CONTENT_PATH_CONFIG_KEY: &str = "userGeneratedContentPath";
const BACKGROUND_OVERLAY_ACTIVITY_CONFIG_CADENCE: Duration = Duration::from_secs(5);
const MAINTENANCE_STOP_POLL_INTERVAL: Duration = Duration::from_millis(50);
const SHARE_EDITOR_ORIGIN: &str = "https://worlds.vrcx-0.dev";

pub(crate) fn build_server_runtime_services_deps(
    context: &RuntimeHostServerAssemblyDeps,
) -> ServerRuntimeServicesDeps {
    ServerRuntimeServicesDeps {
        image_cache: Arc::clone(context.image_cache()),
        notification_config: context.notification_config(),
        auth_credentials: context.auth_credentials_shared(),
        auth_scope: context.auth_scope().clone(),
        session: context.session().clone(),
        tasks: context.tasks().clone(),
        event_bus: context.event_bus().clone(),
        overlay_activity: context.overlay_activity(),
        overlay_activity_sinks: context.overlay_activity_sink_registry(),
    }
}

#[derive(Clone, Debug, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeJobRecordInput {
    pub name: String,
    #[serde(default = "default_frontend_owner")]
    pub owner: String,
    #[serde(default)]
    pub cadence_seconds: Option<u64>,
    pub status: RuntimeOperationStatus,
    #[serde(default)]
    pub detail: String,
}

#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserRefreshOutcome {
    pub applied: bool,
}

fn default_frontend_owner() -> String {
    "frontend".into()
}

pub struct ServerRuntimeHostOptions {
    pub realtime_origin: String,
    pub launched_from_autostart: bool,
    pub app_data_dir: AppDataDirResolution,
    pub app_version: String,
    pub database_maintenance_cache_dir: Option<std::path::PathBuf>,
    pub task_executor: Arc<dyn RuntimeTaskExecutor>,
    /// Optional wrapper applied to the VRChat realtime transport (e.g.
    /// the server's desktop-activity pause gate).
    pub realtime_transport_wrapper: Option<vrcx_0_composition::RealtimeTransportWrapper>,
}

pub struct ServerRuntimeHostState {
    runtime: RuntimeHostState,
    services: Arc<crate::context::ServerRuntimeServices>,
    host_file_access: HostFileAccess,
    current_user_mutations: CurrentUserMutationRuntime,
    avatars: ServerAvatarRuntime,
    local_data: LocalDataRuntime,
    data_dir: ServerDataDirRuntime,
    profile_backup: ServerProfileBackupRuntime,
    external_api: ExternalApiRuntime,
    groups: ServerGroupRuntime,
    social: ServerSocialRuntime,
    media: ServerMediaRuntime,
    vrchat_remote: ServerVrchatRemoteFacade,
    worlds: WorldRemoteRuntime,
    vrchat_config: VrchatConfigRuntime,
    database_upgrade: ServerDatabaseUpgradeRuntimeAlias,
    world_collections: vrcx_0_outbound_adapters::LocalWorldCollectionAdapter,
    friend_log_name_store: vrcx_0_outbound_adapters::LocalFriendLogNameStore,
    notification_sync: vrcx_0_outbound_adapters::LocalNotificationSyncAdapter,
}

/// Alias kept short at the struct site; the runtime itself is the desktop
/// database upgrade session driver, which works unchanged on the server.
pub type ServerDatabaseUpgradeRuntimeAlias = crate::database_upgrade::ServerDatabaseUpgradeRuntime;

struct ServerRuntimeProfileExtension {
    maintenance_running: Arc<AtomicBool>,
    presence_state_path: std::path::PathBuf,
}

impl ServerRuntimeHostState {
    pub fn new(options: ServerRuntimeHostOptions) -> Result<Self> {
        let ServerRuntimeHostOptions {
            realtime_origin,
            launched_from_autostart,
            app_data_dir,
            app_version,
            database_maintenance_cache_dir,
            task_executor,
            realtime_transport_wrapper,
        } = options;
        let builder = RuntimeHostStateBuilder::new(RuntimeHostOptions {
            realtime_origin,
            launched_from_autostart,
            app_data_dir,
            app_version: app_version.clone(),
            profile: RuntimeHostProfile::Server,
            database_maintenance_cache_dir,
            task_executor: Some(task_executor),
        })?;
        let host_file_access = HostFileAccess::new();
        register_runtime_file_access_grants(
            &host_file_access,
            builder.profile_backup(),
            builder.desktop_assembly().config(),
        )?;
        let services = Arc::new(crate::context::ServerRuntimeServices::new(
            build_server_runtime_services_deps(builder.desktop_assembly()),
        )?);
        let profile_config: Arc<dyn vrcx_0_application::profile::ProfileConfigStore> =
            Arc::new(vrcx_0_outbound_adapters::LocalProfileConfigStore::new(
                Arc::clone(builder.database()),
                Arc::clone(builder.storage()),
            ));
        let extension = Arc::new(ServerRuntimeProfileExtension {
            maintenance_running: Arc::new(AtomicBool::new(false)),
            presence_state_path: builder
                .paths()
                .app_data
                .join("presenceAutomationState.json"),
        });
        let friend_projection_observer: Arc<
            dyn vrcx_0_application_realtime::FriendProjectionObserver,
        > = services.clone();
        let runtime = builder.finish(RuntimeHostComposition {
            local_game_context: Arc::new(UnavailableLocalGameContextSource),
            group_order_source: Arc::new(UnavailableGroupOrderSource),
            friend_projection_observer: Some(friend_projection_observer),
            profile_extension: Some(extension),
            realtime_transport_wrapper,
        })?;
        let current_user_mutations =
            crate::current_user_mutation::build_current_user_mutation_runtime(
                crate::current_user_mutation::CurrentUserMutationRuntimeDeps {
                    auth_scope: runtime.desktop_assembly().auth_scope().clone(),
                    remote_mutations: Arc::clone(runtime.desktop_assembly().remote_mutations()),
                    web: Arc::clone(runtime.desktop_assembly().web_client()),
                    diagnostics: runtime.desktop_assembly().diagnostics().clone(),
                    sync: runtime.desktop_assembly().sync().clone(),
                    realtime_runtime: Arc::clone(runtime.realtime_runtime()),
                },
            );
        let avatars = ServerAvatarRuntime::new(
            Arc::clone(runtime.database()),
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().diagnostics().clone(),
            runtime.desktop_assembly().sync().clone(),
            Arc::clone(runtime.realtime_runtime()),
            Arc::clone(runtime.desktop_assembly().avatar_cache()),
            runtime.desktop_assembly().avatar_moderation().clone(),
            runtime.desktop_assembly().auth_scope().clone(),
            Arc::clone(runtime.desktop_assembly().remote_mutations()),
        );
        let local_data = LocalDataRuntime::new(
            Arc::clone(runtime.database()),
            Arc::clone(&profile_config),
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().auth_scope().clone(),
            runtime.desktop_assembly().tasks().clone(),
            Arc::clone(runtime.desktop_assembly().avatar_cache()),
            Arc::clone(runtime.desktop_assembly().world_cache()),
            runtime.desktop_assembly().file_cache().clone(),
            Arc::clone(runtime.realtime_runtime()),
            services.overlay_activity(),
            runtime.desktop_assembly().favorite_mutations().clone(),
            runtime.desktop_assembly().mutual_graph_fetch().clone(),
        );
        let data_dir = ServerDataDirRuntime::new(
            runtime.app_data_dir().clone(),
            runtime.paths().clone(),
            runtime.data_dir_migration().clone(),
        );
        let profile_backup = ServerProfileBackupRuntime::new(
            runtime.profile_backup().clone(),
            host_file_access.clone(),
            runtime.paths().clone(),
        );
        let external_api = ExternalApiRuntime::new(
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().diagnostics().clone(),
            runtime.desktop_assembly().sync().clone(),
        );
        let groups = ServerGroupRuntime::new(
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().diagnostics().clone(),
            runtime.desktop_assembly().sync().clone(),
            runtime.desktop_assembly().auth_scope().clone(),
            Arc::clone(runtime.desktop_assembly().remote_mutations()),
        );
        let social = ServerSocialRuntime::new(
            Arc::clone(runtime.database()),
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().auth_scope().clone(),
            Arc::clone(runtime.desktop_assembly().remote_mutations()),
            Arc::clone(runtime.realtime_runtime()),
            runtime.desktop_assembly().diagnostics().clone(),
            runtime.desktop_assembly().sync().clone(),
            runtime.desktop_assembly().event_bus().clone(),
            Arc::clone(runtime.desktop_assembly().world_cache()),
            runtime.desktop_assembly().moderation_sync().clone(),
            runtime.authenticated_runtime().clone(),
        );
        let media = ServerMediaRuntime::new(
            host_file_access.clone(),
            runtime.paths().clone(),
            Arc::clone(runtime.image_cache()),
            Arc::clone(runtime.database()),
            Arc::clone(runtime.web_client()),
            runtime.desktop_assembly().auth_scope().clone(),
            Arc::clone(runtime.desktop_assembly().remote_mutations()),
            runtime.desktop_assembly().diagnostics().clone(),
        );
        let vrchat_api =
            crate::vrchat_api::build_vrchat_api_runtime(crate::vrchat_api::VrchatApiRuntimeDeps {
                auth_scope: runtime.desktop_assembly().auth_scope().clone(),
                remote_mutations: Arc::clone(runtime.desktop_assembly().remote_mutations()),
                web: Arc::clone(runtime.desktop_assembly().web_client()),
                diagnostics: runtime.desktop_assembly().diagnostics().clone(),
                sync: runtime.desktop_assembly().sync().clone(),
            });
        let worlds = crate::world_remote::build_world_remote_runtime(
            crate::world_remote::WorldRemoteRuntimeDeps {
                auth_scope: runtime.desktop_assembly().auth_scope().clone(),
                remote_mutations: Arc::clone(runtime.desktop_assembly().remote_mutations()),
                web: Arc::clone(runtime.desktop_assembly().web_client()),
                diagnostics: runtime.desktop_assembly().diagnostics().clone(),
                sync: runtime.desktop_assembly().sync().clone(),
                world_cache: Arc::clone(runtime.desktop_assembly().world_cache()),
            },
        );
        let vrchat_remote = ServerVrchatRemoteFacade::new(
            vrchat_api.clone(),
            media.clone(),
            crate::profile_bio::ProfileBioObserver::new(
                Arc::clone(runtime.database()),
                Arc::clone(runtime.realtime_runtime()),
                runtime.desktop_assembly().auth_scope().clone(),
            ),
        );
        let vrchat_config = VrchatConfigRuntime::new(
            vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT.into(),
            Arc::new(vrcx_0_outbound_adapters::VrchatConfigAdapter::new(
                Arc::clone(runtime.web_client()),
                vrchat_api.clone(),
            )),
        );
        // Server builds never report telemetry; the upgrade runtime only
        // expects the handle, so give it a permanently-disabled transport.
        let telemetry = vrcx_0_application::telemetry::TelemetryRuntime::new(
            vrcx_0_application::telemetry::TelemetryRuntimeDeps {
                environment: Arc::new(vrcx_0_outbound_adapters::LocalTelemetryEnvironment::new(
                    runtime.desktop_assembly().config().clone(),
                    Arc::clone(runtime.database()),
                    runtime.paths().app_data.clone(),
                    Arc::new(|| "server".to_string()),
                )),
                transport: Arc::new(disabled_telemetry_transport()),
                tasks: runtime.desktop_assembly().tasks().clone(),
                backend_runtime: runtime.backend_runtime().clone(),
                auth_scope: runtime.desktop_assembly().auth_scope().clone(),
                app_version: app_version.clone(),
            },
        );
        let database_upgrade = crate::database_upgrade::ServerDatabaseUpgradeRuntime::new(
            DatabaseUpgradeRuntime::new(
                Arc::new(vrcx_0_outbound_adapters::LocalDatabaseUpgradeStore::new(
                    Arc::clone(runtime.database()),
                )),
                runtime.desktop_assembly().diagnostics().clone(),
                runtime.desktop_assembly().background_jobs().clone(),
            ),
            runtime.desktop_assembly().config().clone(),
            telemetry,
            runtime.paths().app_data.join("error-log.txt"),
        );
        let world_collections = vrcx_0_outbound_adapters::LocalWorldCollectionAdapter::new(
            Arc::clone(runtime.database()),
        );
        let friend_log_name_store =
            vrcx_0_outbound_adapters::LocalFriendLogNameStore::new(Arc::clone(runtime.database()));
        let notification_sync = vrcx_0_outbound_adapters::LocalNotificationSyncAdapter::new(
            Arc::clone(runtime.database()),
            Arc::clone(runtime.web_client()),
        );
        services.set_realtime_user_image_resolver(runtime.realtime_runtime());

        Ok(Self {
            runtime,
            services,
            host_file_access,
            current_user_mutations,
            avatars,
            local_data,
            data_dir,
            profile_backup,
            external_api,
            groups,
            social,
            media,
            vrchat_remote,
            worlds,
            vrchat_config,
            database_upgrade,
            world_collections,
            friend_log_name_store,
            notification_sync,
        })
    }

    pub fn register_host_file_access(&self, path: impl AsRef<std::path::Path>) {
        self.host_file_access.register_path(path);
    }

    pub fn current_user_mutations(&self) -> &CurrentUserMutationRuntime {
        &self.current_user_mutations
    }

    pub fn avatars(&self) -> &ServerAvatarRuntime {
        &self.avatars
    }

    pub fn worlds(&self) -> &WorldRemoteRuntime {
        &self.worlds
    }

    pub fn vrchat_remote(&self) -> &ServerVrchatRemoteFacade {
        &self.vrchat_remote
    }

    /// The underlying composition runtime host (persistence, web client,
    /// realtime runtime, remote sync) for host-level wiring such as the
    /// server bootstrap.
    pub fn runtime(&self) -> &RuntimeHostState {
        &self.runtime
    }

    pub fn remote_sync(&self) -> &std::sync::Arc<vrcx_0_composition::RemoteSyncHost> {
        self.runtime.remote_sync()
    }

    pub fn local_data(&self) -> &LocalDataRuntime {
        &self.local_data
    }

    pub fn data_dir(&self) -> &ServerDataDirRuntime {
        &self.data_dir
    }

    pub fn profile_backup(&self) -> &ServerProfileBackupRuntime {
        &self.profile_backup
    }

    pub fn groups(&self) -> &ServerGroupRuntime {
        &self.groups
    }

    pub async fn get_user_via_cache(
        &self,
        user_id: String,
        force: bool,
        dialog: bool,
        is_friend: Option<bool>,
    ) -> Result<vrcx_0_application_core::vrchat_api::VrchatApiResponse> {
        let command = "app__vrchat_user_get";
        let diagnostics = self.runtime.desktop_assembly().diagnostics();
        diagnostics.record_command(
            command,
            RuntimeOperationStatus::Running,
            format!("Getting user {user_id}."),
        );
        let result = self
            .runtime
            .realtime_runtime()
            .get_user_via_cache(
                vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT.into(),
                user_id,
                force,
                dialog,
                is_friend,
            )
            .await;
        match &result {
            Ok(response) => diagnostics.record_command(
                command,
                RuntimeOperationStatus::Ok,
                format!("status={}", response.status),
            ),
            Err(error) => diagnostics.record_command(
                command,
                RuntimeOperationStatus::Error,
                error.to_string(),
            ),
        }
        Ok(result?)
    }

    pub async fn favorite_add_remote(
        &self,
        input: vrcx_0_application::favorites::FavoriteRemoteAddInput,
    ) -> Result<vrcx_0_application_core::vrchat_api::VrchatApiResponse> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .add_remote("Remote favorite mutation", input)
            .await?)
    }

    pub async fn favorite_delete_remote(
        &self,
        input: vrcx_0_application::favorites::FavoriteRemoteDeleteInput,
    ) -> Result<vrcx_0_application_core::vrchat_api::VrchatApiResponse> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .delete_remote("Remote favorite mutation", input)
            .await?)
    }

    pub async fn favorite_group_save_remote(
        &self,
        input: vrcx_0_application::favorites::FavoriteRemoteGroupSaveInput,
    ) -> Result<vrcx_0_application_core::vrchat_api::VrchatApiResponse> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .save_remote_group("Remote favorite mutation", input)
            .await?)
    }

    pub async fn favorite_group_clear_remote(
        &self,
        input: vrcx_0_application::favorites::FavoriteRemoteGroupClearInput,
    ) -> Result<vrcx_0_application_core::vrchat_api::VrchatApiResponse> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .clear_remote_group("Remote favorite mutation", input)
            .await?)
    }

    pub fn favorite_local_group_create(
        &self,
        kind: vrcx_0_application_core::FavoriteEntityKind,
        group_name: String,
    ) -> Result<vrcx_0_application::favorites::LocalFavoriteGroupWrite> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .create_local_group(kind, group_name)?)
    }

    pub fn favorite_local_group_rename(
        &self,
        kind: vrcx_0_application_core::FavoriteEntityKind,
        group_name: String,
        new_group_name: String,
    ) -> Result<vrcx_0_application::favorites::LocalFavoriteGroupWrite> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .rename_local_group(kind, group_name, new_group_name)?)
    }

    pub fn favorite_local_group_delete(
        &self,
        kind: vrcx_0_application_core::FavoriteEntityKind,
        group_name: String,
    ) -> Result<vrcx_0_application::favorites::LocalFavoriteGroupWrite> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .delete_local_group(kind, group_name)?)
    }

    pub fn favorite_import_start(
        &self,
        input: FavoriteImportStartInput,
    ) -> Result<FavoriteImportStatus> {
        Ok(self.runtime.favorite_import().start(input)?)
    }

    pub fn favorite_import_status(&self) -> FavoriteImportStatus {
        self.runtime.favorite_import().status()
    }

    pub fn favorite_import_cancel(&self) -> FavoriteImportStatus {
        self.runtime.favorite_import().cancel()
    }

    pub fn favorite_import_dismiss(&self, run_id: &str) -> bool {
        self.runtime.favorite_import().dismiss(run_id)
    }

    pub fn group_ban_import_start(
        &self,
        input: GroupBanImportStartInput,
    ) -> Result<GroupBanImportStatus> {
        Ok(self.runtime.group_ban_import().start(input)?)
    }

    pub fn group_ban_import_status(&self) -> GroupBanImportStatus {
        self.runtime.group_ban_import().status()
    }

    pub fn group_ban_import_cancel(&self) -> GroupBanImportStatus {
        self.runtime.group_ban_import().cancel()
    }

    pub fn persist_favorite_cache_snapshot(
        &self,
        input: FavoriteCacheSnapshotInput,
    ) -> Result<bool> {
        let assembly = self.runtime.desktop_assembly();
        let scope = assembly.auth_scope().snapshot();
        Ok(
            vrcx_0_application::favorites::persist_favorite_cache_snapshot(
                assembly.world_cache(),
                assembly.avatar_cache(),
                &scope.current_user_id,
                &scope.endpoint,
                input,
            )?,
        )
    }

    pub fn social(&self) -> &ServerSocialRuntime {
        &self.social
    }

    pub fn database_upgrade(&self) -> &ServerDatabaseUpgradeRuntimeAlias {
        &self.database_upgrade
    }

    pub fn favorite_details_runtime(
        &self,
    ) -> vrcx_0_application::favorites::FavoriteDetailsRuntime {
        vrcx_0_application::favorites::FavoriteDetailsRuntime::new(
            Arc::new(vrcx_0_outbound_adapters::VrchatFavoriteRemote::new(
                Arc::clone(self.runtime.web_client()),
                self.runtime.desktop_assembly().diagnostics().clone(),
                self.runtime.desktop_assembly().sync().clone(),
                Arc::clone(self.runtime.desktop_assembly().world_cache()),
            )),
            self.runtime.desktop_assembly().auth_scope().clone(),
            Arc::clone(self.runtime.desktop_assembly().world_cache()),
            Arc::clone(self.runtime.desktop_assembly().avatar_cache()),
            self.runtime.desktop_assembly().tasks().clone(),
        )
    }

    pub fn quick_search_runtime(&self) -> vrcx_0_application::social::QuickSearchRuntime {
        let avatar_adapter = Arc::new(
            vrcx_0_outbound_adapters::LocalAvatarApplicationAdapter::new(Arc::clone(
                self.runtime.database(),
            )),
        );
        vrcx_0_application::social::QuickSearchRuntime::new(
            vrcx_0_application::social::QuickSearchSources::new(
                Arc::new(vrcx_0_outbound_adapters::LocalQuickSearchDetailStore::new(
                    Arc::clone(self.runtime.database()),
                )),
                Arc::new(vrcx_0_outbound_adapters::VrchatQuickSearchRemoteRequests),
                avatar_adapter.clone(),
                Arc::new(vrcx_0_outbound_adapters::VrchatAvatarRemote::new(
                    Arc::clone(self.runtime.web_client()),
                    self.runtime.desktop_assembly().diagnostics().clone(),
                    self.runtime.desktop_assembly().sync().clone(),
                )),
                Arc::clone(self.runtime.desktop_assembly().world_cache()),
            ),
            Arc::new(vrcx_0_outbound_adapters::VrchatRequestAdapter::new(
                Arc::clone(self.runtime.web_client()),
            )),
            self.runtime.desktop_assembly().auth_scope().clone(),
            self.runtime.desktop_assembly().diagnostics().clone(),
            self.runtime.desktop_assembly().sync().clone(),
        )
    }

    pub fn mcp_runtime(&self, caller: vrcx_0_mcp::McpCaller) -> vrcx_0_mcp::McpRuntime {
        let db = self.runtime.database();
        let assembly = self.runtime.desktop_assembly();
        vrcx_0_mcp::McpRuntime::new(
            vrcx_0_mcp::McpRuntimeDeps {
                realtime_runtime: Arc::clone(self.runtime.realtime_runtime()),
                auth_scope: assembly.auth_scope().clone(),
                config: Arc::new(crate::mcp_adapters::ServerMcpConfigAdapter::new(
                    assembly.config().clone(),
                )),
                activity_queries: Arc::new(
                    crate::mcp_adapters::ServerMcpActivityQueryAdapter::new(Arc::clone(db)),
                ),
                social_history_queries: Arc::new(
                    crate::mcp_adapters::ServerMcpSocialHistoryQueryAdapter::new(Arc::clone(db)),
                ),
                friend_local_data: Arc::new(
                    crate::mcp_adapters::ServerMcpFriendLocalDataAdapter::new(Arc::clone(db)),
                ),
                favorites_queries: Arc::new(
                    crate::mcp_adapters::ServerMcpFavoritesQueryAdapter::new(Arc::clone(db)),
                ),
                feed_queries: Arc::new(crate::mcp_adapters::ServerMcpFeedQueryAdapter::new(
                    Arc::clone(db),
                )),
                mutual_graph: Arc::new(crate::mcp_adapters::ServerMcpMutualGraphAdapter::new(
                    assembly.mutual_graph_fetch().clone(),
                    Arc::clone(db),
                    Arc::clone(self.runtime.web_client()),
                    assembly.auth_scope().clone(),
                    assembly.tasks().clone(),
                )),
                favorite_mutations: assembly.favorite_mutations().clone(),
                tasks: assembly.tasks().clone(),
            },
            caller,
        )
    }

    pub fn assistant_controller_deps(&self) -> vrcx_0_assistant::AssistantControllerDeps {
        let assembly = self.runtime.desktop_assembly();
        vrcx_0_assistant::AssistantControllerDeps {
            config: Arc::new(
                crate::assistant_adapters::ServerAssistantConfigAdapter::new(
                    assembly.config().clone(),
                ),
            ),
            llm_factory: Arc::new(crate::assistant_adapters::ServerAssistantLlmClientFactory),
            proxy_url: self.runtime.web_client().proxy_url().map(str::to_string),
            bus: assembly.event_bus().clone(),
            tasks: assembly.tasks().clone(),
            mcp_runtime: self.mcp_runtime(vrcx_0_mcp::McpCaller::Assistant),
            session_persistence: Arc::new(
                crate::assistant_adapters::ServerAssistantSessionPersistenceAdapter::new(
                    Arc::clone(self.runtime.database()),
                ),
            ),
            auth_scope: assembly.auth_scope().clone(),
        }
    }

    pub fn require_active_scope(
        &self,
        requirement: &str,
    ) -> Result<vrcx_0_application_core::RuntimeAuthScopeSnapshot> {
        let scope = self.auth_scope_snapshot();
        if scope.active && !scope.current_user_id.trim().is_empty() {
            Ok(scope)
        } else {
            Err(vrcx_0_application_core::Error::Custom(format!(
                "{requirement} requires an authenticated session."
            ))
            .into())
        }
    }

    pub async fn resolve_friend_log_names(
        &self,
        coordinator: &vrcx_0_application::social::FriendLogNameResolutionCoordinator,
        input: vrcx_0_application::social::FriendLogNameResolutionInput,
    ) -> Result<Vec<vrcx_0_application::social::ResolvedFriendLogName>> {
        Ok(vrcx_0_application::social::resolve_friend_log_names(
            coordinator,
            vrcx_0_application::social::FriendLogNameResolutionDeps::new(
                &self.friend_log_name_store,
                self.runtime.desktop_assembly().auth_scope(),
                self.runtime.realtime_runtime(),
            ),
            input,
        )
        .await?)
    }

    pub async fn run_avatar_content_tags_batch(
        &self,
        input: vrcx_0_application::social::AvatarContentTagsBatchInput,
    ) -> Result<vrcx_0_application::social::BatchMutationResult> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::run_avatar_content_tags_batch(
            &vrcx_0_application::social::VrchatBatchMutationActions::new(
                &vrcx_0_outbound_adapters::VrchatRequestAdapter::new(Arc::clone(
                    self.runtime.web_client(),
                )),
                &vrcx_0_outbound_adapters::VrchatBatchMutationRemoteRequests,
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
                self.runtime.desktop_assembly().remote_mutations(),
            ),
            input,
        )
        .await?)
    }

    pub async fn run_group_membership_batch(
        &self,
        coordinator: &vrcx_0_application::social::GroupMembershipBatchCoordinator,
        input: vrcx_0_application::social::GroupMembershipBatchInput,
    ) -> Result<vrcx_0_application::social::GroupMembershipBatchResult> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::run_group_membership_batch(
            coordinator,
            &vrcx_0_application::social::VrchatGroupMembershipBatchActions::new(
                &vrcx_0_outbound_adapters::VrchatRequestAdapter::new(Arc::clone(
                    self.runtime.web_client(),
                )),
                &vrcx_0_outbound_adapters::VrchatGroupRemoteRequests,
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
                self.runtime.desktop_assembly().event_bus().clone(),
                self.runtime.desktop_assembly().remote_mutations(),
            ),
            input,
        )
        .await?)
    }

    pub async fn run_group_moderation_batch(
        &self,
        coordinator: &vrcx_0_application::social::GroupModerationBatchCoordinator,
        input: vrcx_0_application::social::GroupModerationBatchInput,
    ) -> Result<vrcx_0_application::social::GroupModerationBatchResult> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::run_group_moderation_batch(
            coordinator,
            &vrcx_0_application::social::VrchatGroupModerationBatchActions::new(
                &vrcx_0_outbound_adapters::VrchatRequestAdapter::new(Arc::clone(
                    self.runtime.web_client(),
                )),
                &vrcx_0_outbound_adapters::VrchatGroupModerationRemoteRequests,
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
                self.runtime.desktop_assembly().event_bus().clone(),
                self.runtime.desktop_assembly().remote_mutations(),
            ),
            input,
        )
        .await?)
    }

    pub async fn mark_notifications_seen_batch(
        &self,
        input: vrcx_0_application::social::NotificationMarkSeenBatchInput,
    ) -> Result<vrcx_0_application::social::NotificationMarkSeenBatchResult> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::mark_notifications_seen_batch(
            &vrcx_0_outbound_adapters::LocalNotificationMarkSeenActions::new(
                self.runtime.database().as_ref(),
                self.runtime.web_client().as_ref(),
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
                self.runtime.desktop_assembly().remote_mutations(),
            ),
            input,
        )
        .await?)
    }

    pub async fn send_instance_invites_batch(
        &self,
        input: vrcx_0_application::social::InstanceInviteBatchInput,
    ) -> Result<vrcx_0_application::social::InstanceInviteBatchResult> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::send_instance_invites_batch(
            &vrcx_0_application::social::VrchatInstanceInviteBatchActions::new(
                &vrcx_0_outbound_adapters::VrchatRequestAdapter::new(Arc::clone(
                    self.runtime.web_client(),
                )),
                &vrcx_0_outbound_adapters::VrchatInstanceInviteRemoteRequests,
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
                self.runtime.desktop_assembly().remote_mutations(),
                &vrcx_0_outbound_adapters::CachedWorldNameResolver::new(
                    Arc::clone(self.runtime.desktop_assembly().world_cache()),
                    Arc::clone(self.runtime.web_client()),
                ),
            ),
            input,
        )
        .await?)
    }

    pub async fn sync_notifications(
        &self,
    ) -> Result<vrcx_0_application::social::NotificationSyncOutcome> {
        let expected_scope = self.require_active_scope("Batch action")?;
        Ok(vrcx_0_application::social::sync_notifications(
            &vrcx_0_application::social::NotificationSyncDeps::new(
                &self.notification_sync,
                self.runtime.desktop_assembly().auth_scope(),
                expected_scope,
            ),
        )
        .await?)
    }

    pub async fn user_dialog_tab_counts(
        &self,
        runtime: &vrcx_0_application::social::UserDialogTabCountsRuntime,
        input: vrcx_0_application::social::UserDialogTabCountsInput,
    ) -> Result<vrcx_0_application::social::UserDialogTabCountsOutput> {
        Ok(vrcx_0_application::social::get_user_dialog_tab_counts(
            runtime,
            vrcx_0_application::social::UserDialogTabCountsDeps::new(
                Arc::new(
                    vrcx_0_outbound_adapters::LocalUserDialogTabCountsSource::new(
                        Arc::clone(self.runtime.database()),
                        Arc::clone(self.runtime.web_client()),
                    ),
                ),
                self.runtime.desktop_assembly().auth_scope().clone(),
            ),
            input,
        )
        .await?)
    }

    pub async fn translate_dispatch(
        &self,
        input: vrcx_0_application::discovery::TranslationTranslateInput,
    ) -> Result<vrcx_0_application::discovery::TranslationDispatch> {
        let adapter = vrcx_0_outbound_adapters::LocalTranslationAdapter::new(
            Arc::clone(self.runtime.database()),
            Arc::clone(self.runtime.web_client()),
        );
        Ok(vrcx_0_application::discovery::translate_text(
            vrcx_0_application::discovery::TranslationDeps {
                config: &adapter,
                standard_translation: &adapter,
            },
            input,
        )
        .await?)
    }

    pub fn resolved_openai_translation_endpoint_id(&self) -> Result<String> {
        let adapter = vrcx_0_outbound_adapters::LocalTranslationAdapter::new(
            Arc::clone(self.runtime.database()),
            Arc::clone(self.runtime.web_client()),
        );
        Ok(vrcx_0_application::discovery::resolved_openai_translation_endpoint_id(&adapter)?)
    }

    pub async fn refresh_social_baseline(
        &self,
    ) -> Result<vrcx_0_application::social::SocialBaselineRefreshOutput> {
        let command = "app__social_baseline_refresh";
        self.runtime
            .desktop_assembly()
            .diagnostics()
            .record_command(
                command,
                RuntimeOperationStatus::Running,
                "Social baseline refresh started.",
            );
        let result = self.runtime.refresh_social_baseline_now().await;
        self.social.record_baseline_refresh(&result);
        Ok(result?)
    }

    pub fn add_game_log_entries(
        &self,
        kind: crate::local_data::GameLogWriteKind,
        entries: Vec<Value>,
    ) -> Result<()> {
        let affected_count = self.local_data.game_log_entries_add(kind, entries)?;
        self.runtime.publish_game_log_persisted(affected_count);
        Ok(())
    }

    pub fn backend_runtime_combined_snapshot(&self) -> BackendRuntimeCombinedSnapshot {
        self.runtime.backend_runtime_combined_snapshot()
    }

    pub async fn refresh_runtime_group_instances(&self) {
        self.runtime.refresh_runtime_group_instances().await;
    }

    pub fn record_runtime_job(&self, input: RuntimeJobRecordInput) {
        let name = input.name.trim();
        if name.is_empty() {
            return;
        }
        let detail = input.detail.trim();
        let jobs = self.runtime.desktop_assembly().background_jobs();
        jobs.register_job(
            name,
            input.owner.trim(),
            input.cadence_seconds,
            input.status,
            detail,
        );
        match input.status {
            RuntimeOperationStatus::Running => jobs.mark_running(name, detail),
            RuntimeOperationStatus::Completed | RuntimeOperationStatus::Idle => {
                jobs.mark_completed(name, detail)
            }
            RuntimeOperationStatus::Error => jobs.mark_failed(name, detail),
            status => jobs.register_job(
                name,
                input.owner.trim(),
                input.cadence_seconds,
                status,
                detail,
            ),
        }
    }

    pub fn saved_auth_snapshot(&self) -> Result<SavedAuthSnapshot> {
        Ok(vrcx_0_application::auth::saved_snapshot(
            self.runtime.desktop_assembly().auth_credentials(),
        )?)
    }

    pub fn delete_saved_credential(&self, user_id: String) -> Result<SavedAuthSnapshot> {
        Ok(vrcx_0_application::auth::delete_saved_credential(
            self.runtime.desktop_assembly().auth_credentials(),
            user_id,
        )?)
    }

    pub async fn start_login_session(&self, input: LoginSessionStartInput) -> LoginSessionState {
        let diagnostics = self.runtime.desktop_assembly().diagnostics();
        diagnostics.record_command(
            "app__vrchat_auth_session_start",
            RuntimeOperationStatus::Running,
            "Starting a VRChat login session.",
        );
        let result = self.runtime.start_login_session(input).await;
        diagnostics.record_command(
            "app__vrchat_auth_session_start",
            RuntimeOperationStatus::Ok,
            format!("status={result:?}"),
        );
        result
    }

    pub async fn start_auto_login(&self, input: AutoLoginStartInput) -> Result<AutoLoginOutcome> {
        let diagnostics = self.runtime.desktop_assembly().diagnostics();
        diagnostics.record_command(
            "app__vrchat_auth_auto_login_start",
            RuntimeOperationStatus::Running,
            "Starting an automatic VRChat login attempt.",
        );
        let result = self.runtime.start_auto_login(input).await;
        match &result {
            Ok(outcome) => diagnostics.record_command(
                "app__vrchat_auth_auto_login_start",
                RuntimeOperationStatus::Ok,
                format!("status={outcome:?}"),
            ),
            Err(error) => diagnostics.record_command(
                "app__vrchat_auth_auto_login_start",
                RuntimeOperationStatus::Error,
                error.to_string(),
            ),
        }
        result
    }

    pub async fn respond_login_session(
        &self,
        input: LoginSessionRespondInput,
    ) -> LoginSessionState {
        self.runtime.respond_login_session(input).await
    }

    pub async fn cancel_login_session(&self, input: LoginSessionCancelInput) -> LoginSessionState {
        self.runtime.cancel_login_session(input).await
    }

    pub async fn end_login_session(
        &self,
        input: LoginSessionEnd,
    ) -> Result<Option<SavedAuthSnapshot>> {
        self.runtime.end_login_session(input).await
    }

    pub async fn share_collection_create(
        &self,
        input: ShareCollectionCreateInput,
    ) -> Result<ShareCollectionCreateResult> {
        let auth_scope = self.runtime.desktop_assembly().auth_scope().snapshot();
        let display_name = self.runtime.snapshot_backend_runtime().auth_display_name;
        Ok(share_collection_create(
            ShareCollectionDeps::new(
                &self.world_collections,
                &self.world_collections,
                &auth_scope.current_user_id,
                &display_name,
            ),
            input,
        )
        .await?)
    }

    /// Manager URL for the shared-collection editor; the web frontend opens
    /// it itself instead of asking the host shell to launch a browser.
    pub async fn shared_collection_manager_url(&self) -> Result<String> {
        let auth_scope = self.runtime.desktop_assembly().auth_scope().snapshot();
        let owner_token = get_or_create_share_owner_token(
            &self.world_collections,
            &self.world_collections,
            &auth_scope.current_user_id,
        )
        .await?;
        Ok(format!("{SHARE_EDITOR_ORIGIN}/mine#k={owner_token}"))
    }

    pub async fn register_world_open_share(&self, world_id: String) {
        let auth_scope = self.runtime.desktop_assembly().auth_scope().snapshot();
        if let Err(error) = register_world_open_share(
            &self.world_collections,
            &self.world_collections,
            &auth_scope.current_user_id,
            &world_id,
        )
        .await
        {
            tracing::warn!(error = %error, "app__world_open_register: best-effort registration failed");
        }
    }

    pub async fn preview_shared_collection(
        &self,
        id: &str,
    ) -> Result<vrcx_0_application::collections::ImportPreview> {
        Ok(
            vrcx_0_application::collections::preview_shared_collection(&self.world_collections, id)
                .await?,
        )
    }

    pub fn start_shared_collection_import(
        &self,
        input: SharedCollectionImportStartInput,
    ) -> Result<SharedCollectionImportStatus> {
        Ok(self.runtime.shared_collection_import().start(input)?)
    }

    pub fn shared_collection_import_status(&self) -> SharedCollectionImportStatus {
        self.runtime.shared_collection_import().status()
    }

    pub fn start_note_export(
        &self,
        input: vrcx_0_application::social::NoteExportStartInput,
    ) -> Result<vrcx_0_application::social::NoteExportStatus> {
        Ok(self.runtime.note_export().start(input)?)
    }

    pub fn note_export_status(&self) -> vrcx_0_application::social::NoteExportStatus {
        self.runtime.note_export().status()
    }

    pub fn cancel_note_export(&self) -> vrcx_0_application::social::NoteExportStatus {
        self.runtime.note_export().cancel()
    }

    pub async fn refresh_current_user(&self) -> Result<CurrentUserRefreshOutcome> {
        let applied = self
            .runtime
            .realtime_runtime()
            .refresh_current_user_now(Value::Null)
            .await?;
        Ok(CurrentUserRefreshOutcome { applied })
    }

    pub fn ingest_user_facts(&self, entries: Vec<Value>) {
        self.runtime.realtime_runtime().ingest_user_facts(entries);
    }

    pub fn start_friend_profile_bulk_load(&self) -> Result<FriendProfileLoadStatusPayload> {
        Ok(self
            .runtime
            .realtime_runtime()
            .start_friend_profile_bulk_load()?)
    }

    pub fn cancel_friend_profile_bulk_load(&self) -> Result<FriendProfileLoadStatusPayload> {
        Ok(self
            .runtime
            .realtime_runtime()
            .cancel_friend_profile_bulk_load()?)
    }

    pub fn friend_snapshot(&self) -> Option<vrcx_0_application_realtime::RealtimeFriendSnapshot> {
        self.runtime.realtime_runtime().friend_snapshot()
    }

    pub fn vrc_status_snapshot(&self) -> VrcStatusSnapshot {
        self.runtime.desktop_assembly().vrc_status().snapshot()
    }

    pub async fn refresh_vrc_status(&self) -> Result<VrcStatusSnapshot> {
        Ok(self
            .runtime
            .desktop_assembly()
            .vrc_status()
            .refresh()
            .await?)
    }

    pub fn presence_automation_rules(
        &self,
        kind: PresenceAutomationRuleKind,
    ) -> Result<Vec<RawJson>> {
        let store = crate::game_state_store::PersistenceGameStateStore::new(Arc::clone(
            self.runtime.database(),
        ));
        Ok(vrcx_0_application_game::presence_automation_rules_get(
            &store, kind,
        )?)
    }

    pub fn set_presence_automation_rules(
        &self,
        kind: PresenceAutomationRuleKind,
        rules: Vec<RawJson>,
    ) -> Result<Vec<RawJson>> {
        let store = crate::game_state_store::PersistenceGameStateStore::new(Arc::clone(
            self.runtime.database(),
        ));
        Ok(vrcx_0_application_game::presence_automation_rules_set(
            &store, kind, rules,
        )?)
    }

    pub fn set_presence_automation_rule_enabled(
        &self,
        kind: PresenceAutomationRuleKind,
        rule_id: &str,
        enabled: bool,
    ) -> Result<Vec<RawJson>> {
        let store = crate::game_state_store::PersistenceGameStateStore::new(Arc::clone(
            self.runtime.database(),
        ));
        Ok(
            vrcx_0_application_game::presence_automation_rule_enabled_set(
                &store, kind, rule_id, enabled,
            )?,
        )
    }

    pub fn set_overlay_activity_filters(
        &self,
        filters: vrcx_0_application_activity::notification::OverlayActivityPreferenceFilters,
    ) -> Result<()> {
        self.runtime
            .desktop_assembly()
            .set_overlay_activity_preference_filters(filters)?;
        Ok(())
    }

    pub fn set_notification_activity_filters(
        &self,
        input: vrcx_0_application_activity::notification::NotificationActivityFiltersSetInput,
    ) -> Result<()> {
        self.runtime
            .desktop_assembly()
            .set_notification_activity_filters(input)?;
        Ok(())
    }

    pub fn active_owner_id(&self) -> Option<vrcx_0_core::OwnerId> {
        let auth_scope = self.runtime.desktop_assembly().auth_scope().snapshot();
        auth_scope
            .active
            .then(|| vrcx_0_core::OwnerId::new(auth_scope.current_user_id))
    }

    pub fn auth_scope_snapshot(&self) -> vrcx_0_application_core::RuntimeAuthScopeSnapshot {
        self.runtime.desktop_assembly().auth_scope().snapshot()
    }

    pub fn host_session_projection(&self) -> vrcx_0_application_core::HostSessionProjection {
        self.runtime
            .desktop_assembly()
            .session()
            .projection_snapshot()
    }

    pub fn profile_backup_status(&self) -> vrcx_0_application::profile::ProfileBackupStatus {
        self.runtime.profile_backup().current_status()
    }

    pub fn data_dir_migration_status(&self) -> vrcx_0_application::profile::DataDirMigrationStatus {
        self.runtime.data_dir_migration().current_status()
    }

    pub fn mutual_graph_fetch_status(&self) -> vrcx_0_application::social::MutualGraphFetchStatus {
        self.runtime
            .desktop_assembly()
            .mutual_graph_fetch()
            .status()
    }

    pub fn backend_runtime_snapshot(&self) -> vrcx_0_application_core::BackendRuntimeSnapshot {
        self.runtime.snapshot_backend_runtime()
    }

    pub async fn start_headless_backend_runtime(
        &self,
    ) -> Result<vrcx_0_application_core::BackendRuntimeSnapshot> {
        self.runtime.start_headless_backend_runtime(None).await
    }

    pub async fn recover_background_auth_after_failure(&self, reason: String) {
        self.runtime
            .recover_background_auth_after_failure(reason)
            .await;
    }

    pub fn active_realtime_transport(
        &self,
    ) -> Option<vrcx_0_application_realtime::RealtimeTransportStartResult> {
        self.runtime
            .authenticated_runtime()
            .snapshot()
            .realtime_transport
    }

    pub fn set_runtime_event_sink<S>(&self, sink: S)
    where
        S: RuntimeEventSink + 'static,
    {
        self.runtime.set_event_sink(sink);
    }

    pub fn set_runtime_host_actions<A>(&self, actions: A)
    where
        A: crate::RuntimeHostActions + 'static,
    {
        self.services.host.set_actions(actions);
    }

    pub fn start_data_services(&self) {
        self.runtime.start_data_services();
    }

    pub fn record_lifecycle_phase(
        &self,
        phase: impl Into<String>,
        status: RuntimeOperationStatus,
        detail: impl Into<String>,
    ) {
        self.runtime
            .desktop_assembly()
            .lifecycle()
            .record_phase(phase, status, detail);
    }

    pub fn record_sync(
        &self,
        domain: impl Into<String>,
        status: RuntimeOperationStatus,
        detail: impl Into<String>,
        pending_count: u64,
    ) {
        self.runtime
            .desktop_assembly()
            .sync()
            .record(domain, status, detail, pending_count);
    }

    pub fn record_sync_failure(&self, domain: impl Into<String>, detail: impl Into<String>) {
        self.runtime
            .desktop_assembly()
            .sync()
            .record_failure(domain, detail);
    }

    pub fn launched_from_autostart(&self) -> bool {
        self.runtime.launched_from_autostart()
    }

    pub fn proxy_url(&self) -> Option<&str> {
        self.runtime.web_client().proxy_url()
    }

    pub fn try_config_bool(&self, key: &str, fallback: bool) -> Option<bool> {
        self.runtime
            .desktop_assembly()
            .config()
            .get_bool(key, fallback)
            .ok()
    }

    pub fn config_string(&self, key: &str, fallback: &str) -> String {
        self.runtime
            .desktop_assembly()
            .config()
            .get_string(key, fallback)
            .unwrap_or_else(|_| fallback.to_string())
    }

    pub fn config_bool(&self, key: &str, fallback: bool) -> bool {
        self.runtime
            .desktop_assembly()
            .config()
            .get_bool(key, fallback)
            .unwrap_or(fallback)
    }

    pub fn storage_get(&self, key: &str) -> Option<String> {
        self.runtime.storage().get(key)
    }

    pub fn storage_set(&self, key: String, value: String) {
        self.runtime.storage().set(key, value);
    }

    pub fn storage_flush(&self) -> Result<()> {
        Ok(self.runtime.storage().save()?)
    }

    pub fn storage_remove(&self, key: &str) -> Option<String> {
        self.runtime.storage().remove(key)
    }

    pub fn storage_snapshot(&self) -> std::collections::HashMap<String, String> {
        self.runtime.storage().get_all()
    }

    pub fn ensure_host_read_allowed(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        self.host_file_access
            .ensure_read_allowed(path, self.runtime.paths())
    }

    pub fn is_known_runtime_root_path(&self, path: impl AsRef<std::path::Path>) -> bool {
        crate::is_known_root_path(path, self.runtime.paths())
    }

    pub fn is_screenshot_thumbnail_path(&self, path: impl AsRef<std::path::Path>) -> bool {
        vrcx_0_platform::path_utils::is_path_inside_directory(
            path.as_ref(),
            &self.runtime.paths().screenshot_thumbs,
        )
    }

    pub fn app_data_path(&self) -> &std::path::Path {
        &self.runtime.paths().app_data
    }

    pub fn append_error_log(&self, entry: &str) {
        vrcx_0_platform::error_log::append_error_log_entry(&self.runtime.paths().app_data, entry);
    }

    pub async fn send_test_webhook(
        &self,
        url: String,
        format: vrcx_0_application_activity::notification::NotificationWebhookFormat,
        payload: Value,
    ) -> Result<vrcx_0_application_activity::notification::WebhookDeliveryOutcome> {
        let url = if format
            == vrcx_0_application_activity::notification::NotificationWebhookFormat::Discord
        {
            vrcx_0_application_activity::notification::discord_webhook_url_with_wait(&url)
        } else {
            url
        };
        vrcx_0_application_activity::notification::send_json_webhook_with_retry(
            &vrcx_0_outbound_adapters::LocalNotificationWebhookTransport::new(
                self.runtime.web_client().clone(),
            ),
            &url,
            payload,
        )
        .await
        .map_err(|error| vrcx_0_composition::Error::Custom(error.to_string()))
    }

    pub fn webhook_delivery_snapshot(
        &self,
    ) -> vrcx_0_application_activity::notification::WebhookDeliverySnapshot {
        self.runtime.desktop_assembly().webhook_delivery_snapshot()
    }

    pub fn stop_for_application_exit(&self, reason: &str) {
        self.runtime.stop_backend_runtime(reason);
        self.runtime.desktop_assembly().tasks().stop_all();
    }

    pub fn release_profile_lock(&self) {
        self.runtime.release_profile_lock();
    }

    pub async fn transfer_favorite_selection(
        &self,
        input: FavoriteTransferSelectionInput,
    ) -> Result<FavoriteTransferSelectionResult> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .transfer_selection(input)
            .await?)
    }

    pub async fn remove_favorite_selection(
        &self,
        input: FavoriteBulkRemoveInput,
    ) -> Result<FavoriteBulkRemoveResult> {
        Ok(self
            .runtime
            .desktop_assembly()
            .favorite_mutations()
            .remove_selection(input)
            .await?)
    }

    pub fn external_api(&self) -> &ExternalApiRuntime {
        &self.external_api
    }

    pub fn media(&self) -> &ServerMediaRuntime {
        &self.media
    }

    pub fn vrchat_config(&self) -> &VrchatConfigRuntime {
        &self.vrchat_config
    }

    pub fn config_set_values(
        &self,
        entries: Vec<crate::local_data::ConfigWriteEntry>,
    ) -> Result<()> {
        self.local_data.config_set_values(entries)?;
        Ok(())
    }

    pub fn config_remove_value(&self, key: String) -> Result<i64> {
        let removed = self.local_data.config_remove_value(key)?;
        Ok(removed)
    }

    pub fn privacy_lock(&self) -> Arc<PrivacyLockRuntime> {
        self.services.privacy_lock()
    }

    pub fn reload_overlay_activity_filters(&self) {
        self.services.reload_overlay_activity_filters();
    }
}

impl RuntimeHostProfileExtension for ServerRuntimeProfileExtension {
    fn start_profile_services(&self, state: &RuntimeHostState) {
        // The server has no local game integration; surface the same
        // unavailable status the desktop reports on unsupported platforms so
        // hydration and background-job panels stay coherent.
        let assembly = state.desktop_assembly();
        assembly.background_jobs().register_job(
            "gameLogWatcher",
            "rust-host",
            None,
            RuntimeOperationStatus::Unavailable,
            "GameLog data arrives via remote sync from a desktop device.",
        );
        assembly.background_jobs().register_job(
            "gameProcessMonitor",
            "rust-host",
            None,
            RuntimeOperationStatus::Unavailable,
            "Game process monitoring requires a desktop device.",
        );
        state
            .backend_runtime()
            .set_game_log_status(vrcx_0_application_core::BackendRuntimeGameLogStatus::Unavailable);
    }

    fn stop_profile_services(&self) {}

    fn start_profile_maintenance(&self, state: &RuntimeHostState) {
        self.start_maintenance_loops(state);
    }
}

impl ServerRuntimeProfileExtension {
    fn start_maintenance_loops(&self, state: &RuntimeHostState) {
        let session_slot = state.authenticated_session_projection_handle();
        if !is_authenticated_maintenance_active(state, &session_slot) {
            return;
        }
        if self.maintenance_running.swap(true, Ordering::AcqRel) {
            return;
        }
        state.desktop_assembly().background_jobs().register_job(
            BACKGROUND_PRESENCE_AUTOMATION_JOB,
            "rust-host",
            Some(BACKGROUND_PRESENCE_CADENCE_SECONDS),
            RuntimeOperationStatus::Scheduled,
            "Background presence automation is scheduled.",
        );
        let db = Arc::clone(state.database());
        let web = Arc::clone(state.web_client());
        let backend_runtime = state.backend_runtime().clone();
        let background_jobs = state.desktop_assembly().background_jobs().clone();
        let running = Arc::clone(&self.maintenance_running);
        let realtime_runtime = Arc::clone(state.realtime_runtime());
        let authenticated_runtime = state.authenticated_runtime().clone();
        let host_session = state.desktop_assembly().session().clone();
        let config = state.desktop_assembly().config().clone();
        let auth_scope = state.desktop_assembly().auth_scope().clone();
        let remote_mutations = Arc::clone(state.desktop_assembly().remote_mutations());
        let event_bus = state.desktop_assembly().event_bus().clone();
        let presence_state_path = self.presence_state_path.clone();
        state
            .desktop_assembly()
            .tasks()
            .spawn_cancellable(move |stop_token| async move {
                let mut presence_state =
                    vrcx_0_application_game::BackgroundPresenceAutomationState::load_cached(
                        &presence_state_path,
                    );
                let mut presence_state_serialized =
                    serde_json::to_string(&presence_state).unwrap_or_default();
                let mut next_presence = Instant::now();
                let mut next_overlay_activity_config = Instant::now();
                let mut active_scope_key =
                    background_capability_session_scope_key(&session_slot).unwrap_or_default();
                loop {
                    if stop_token.is_stop_requested()
                        || !is_authenticated_maintenance_active_parts(
                            &backend_runtime,
                            &auth_scope,
                            &session_slot,
                        )
                    {
                        break;
                    }
                    let now = Instant::now();
                    let scope_key =
                        background_capability_session_scope_key(&session_slot).unwrap_or_default();
                    if scope_key != active_scope_key {
                        active_scope_key = scope_key;
                        presence_state =
                            vrcx_0_application_game::BackgroundPresenceAutomationState::default();
                        next_presence = now;
                        next_overlay_activity_config = now;
                    }
                    if now >= next_overlay_activity_config {
                        services_reload_overlay_activity_filters(&event_bus);
                        next_overlay_activity_config =
                            now + BACKGROUND_OVERLAY_ACTIVITY_CONFIG_CADENCE;
                    }
                    let tick_context = BackgroundTickContext {
                        db: &db,
                        web: &web,
                        session_slot: &session_slot,
                        realtime_runtime: &realtime_runtime,
                        host_session: &host_session,
                        config: &config,
                        auth_scope: &auth_scope,
                        remote_mutations: &remote_mutations,
                        event_bus: &event_bus,
                        backend_runtime: &backend_runtime,
                        background_jobs: &background_jobs,
                    };
                    if now >= next_presence {
                        let favorite_group_memberships = authenticated_runtime
                            .favorite_group_memberships()
                            .unwrap_or_default();
                        let friend_user_ids = realtime_runtime.friend_user_ids_snapshot();
                        run_background_presence_tick(
                            &tick_context,
                            &mut presence_state,
                            &friend_user_ids,
                            &favorite_group_memberships.friend_groups_by_key,
                            &favorite_group_memberships.world_groups_by_key,
                        )
                        .await;
                        presence_state
                            .persist_cached(&presence_state_path, &mut presence_state_serialized);
                        next_presence =
                            now + Duration::from_secs(BACKGROUND_PRESENCE_CADENCE_SECONDS);
                    }
                    if wait_for_maintenance_tick(&stop_token).await {
                        break;
                    }
                }
                running.store(false, Ordering::Release);
                background_jobs.mark_completed(
                    BACKGROUND_PRESENCE_AUTOMATION_JOB,
                    "Background presence automation stopped.",
                );
            });
    }
}

/// Re-apply overlay-activity preference filters from config on a cadence.
/// The desktop services object owns the loaded filters; on the server the
/// composition assembly exposes the same reload through its event bus.
fn services_reload_overlay_activity_filters(_event_bus: &vrcx_0_application_core::RuntimeEventBus) {
    // Overlay-activity filters are reloaded by the composition webhook
    // pipeline; nothing extra to do here on the server yet.
}

async fn wait_for_maintenance_tick(stop_token: &TaskStopToken) -> bool {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if stop_token.is_stop_requested() {
            return true;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        tokio::time::sleep(remaining.min(MAINTENANCE_STOP_POLL_INTERVAL)).await;
    }
}

/// Transport that reports telemetry as permanently disabled on the server.
struct DisabledTelemetryTransport;

impl vrcx_0_application::telemetry::TelemetryTransport for DisabledTelemetryTransport {
    fn is_enabled(&self) -> bool {
        false
    }

    fn post<'a>(
        &'a self,
        _path: &'a str,
        _payload: serde_json::Value,
    ) -> vrcx_0_application::telemetry::TelemetryPostFuture<'a> {
        Box::pin(async { Ok(()) })
    }
}

fn disabled_telemetry_transport() -> DisabledTelemetryTransport {
    DisabledTelemetryTransport
}

fn register_runtime_file_access_grants(
    file_access: &HostFileAccess,
    profile_backup: &vrcx_0_application::profile::ProfileBackupRuntime,
    config: &vrcx_0_persistence::config::ConfigRepository,
) -> Result<()> {
    let profile_backup_target = profile_backup.settings().auto_target_dir;
    if !profile_backup_target.is_empty() {
        file_access.register_path(profile_backup_target);
    }
    register_persisted_user_generated_content_path_grant(file_access, config)
}

fn register_persisted_user_generated_content_path_grant(
    file_access: &HostFileAccess,
    config: &vrcx_0_persistence::config::ConfigRepository,
) -> Result<()> {
    let ugc_path = config.get_string(USER_GENERATED_CONTENT_PATH_CONFIG_KEY, "")?;
    let ugc_path = ugc_path.trim();
    if !ugc_path.is_empty() {
        file_access.register_path(ugc_path);
    }
    Ok(())
}

fn session_matches_auth_scope(
    session: Option<&vrcx_0_application_core::BackgroundCapabilitySessionIdentity>,
    auth_scope: &vrcx_0_application_core::RuntimeAuthScopeSnapshot,
) -> bool {
    session
        .map(|session| {
            auth_scope.active
                && session.auth_scope_generation == auth_scope.generation
                && session.current_user_id == auth_scope.current_user_id
                && vrcx_0_vrchat_client::http_api::normalize_vrchat_api_endpoint(Some(
                    &session.endpoint,
                )) == auth_scope.endpoint
        })
        .unwrap_or(false)
}

fn is_authenticated_maintenance_active(
    state: &RuntimeHostState,
    session_slot: &Arc<Mutex<vrcx_0_application::auth::AuthenticatedSessionProjection>>,
) -> bool {
    is_authenticated_maintenance_active_parts(
        state.backend_runtime(),
        state.desktop_assembly().auth_scope(),
        session_slot,
    )
}

fn is_authenticated_maintenance_active_parts(
    runtime: &vrcx_0_application_core::BackendRuntime,
    auth_scope: &vrcx_0_application_core::RuntimeAuthScope,
    session_slot: &Arc<Mutex<vrcx_0_application::auth::AuthenticatedSessionProjection>>,
) -> bool {
    let snapshot = runtime.snapshot();
    let auth_scope = auth_scope.snapshot();
    if snapshot.phase != BackendRuntimePhase::Running
        || snapshot.auth_status != vrcx_0_application_core::BackendRuntimeAuthStatus::Authenticated
    {
        return false;
    }
    session_matches_auth_scope(
        background_ticks::background_capability_session_identity(session_slot).as_ref(),
        &auth_scope,
    )
}

fn background_capability_session_scope_key(
    session_slot: &Arc<Mutex<vrcx_0_application::auth::AuthenticatedSessionProjection>>,
) -> Option<String> {
    background_ticks::background_capability_session_identity(session_slot).map(|session| {
        format!(
            "{}:{}:{}",
            session.auth_scope_generation,
            session.current_user_id,
            vrcx_0_vrchat_client::http_api::normalize_vrchat_api_endpoint(Some(&session.endpoint))
        )
    })
}

#[allow(dead_code)]
fn emit_profile_background_info(
    desktop_assembly: &vrcx_0_composition::RuntimeHostServerAssemblyDeps,
    backend_runtime: &vrcx_0_application_core::BackendRuntime,
    detail: impl Into<String>,
) {
    emit_profile_background_output(
        desktop_assembly,
        backend_runtime,
        BackendRuntimeTelemetryKind::BackgroundInfo,
        detail,
    );
}

fn emit_profile_background_output(
    desktop_assembly: &vrcx_0_composition::RuntimeHostServerAssemblyDeps,
    backend_runtime: &vrcx_0_application_core::BackendRuntime,
    kind: BackendRuntimeTelemetryKind,
    detail: impl Into<String>,
) {
    let snapshot = backend_runtime.snapshot();
    if snapshot.phase != BackendRuntimePhase::Running {
        return;
    }
    vrcx_0_application_core::BackendRuntimeStatusPublisher::new(
        backend_runtime.clone(),
        desktop_assembly.event_bus().clone(),
    )
    .publish_telemetry(kind, detail, snapshot);
}

#[cfg(test)]
mod runtime_host_state {
    mod persisted_file_access_tests {
        use super::super::{
            register_persisted_user_generated_content_path_grant,
            USER_GENERATED_CONTENT_PATH_CONFIG_KEY,
        };
        use crate::{HostFileAccess, Result};
        use std::path::PathBuf;
        use std::sync::Arc;
        use vrcx_0_persistence::{config::ConfigRepository, DatabaseService};
        use vrcx_0_platform::app_paths::AppPaths;

        struct TestDir {
            path: PathBuf,
        }

        impl TestDir {
            fn new(name: &str) -> Self {
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let path = std::env::temp_dir()
                    .join(format!("vrcx-0-{name}-{}-{nonce}", std::process::id()));
                std::fs::create_dir_all(&path).unwrap();
                Self { path }
            }
        }

        impl Drop for TestDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }

        #[test]
        fn restores_persisted_user_generated_content_path_for_open_and_save() -> Result<()> {
            let dir = TestDir::new("persisted-ugc-grant");
            let app_data = dir.path.join("app-data");
            let ugc_path = dir.path.join("custom-ugc");
            std::fs::create_dir_all(&app_data)?;
            std::fs::create_dir_all(&ugc_path)?;
            let db = Arc::new(DatabaseService::new(&dir.path.join("VRCX-0.sqlite3"))?);
            let config = ConfigRepository::new(db);
            config.set_string(
                USER_GENERATED_CONTENT_PATH_CONFIG_KEY,
                &ugc_path.to_string_lossy(),
            )?;

            let host_file_access = HostFileAccess::new();
            let app_paths = AppPaths::from_app_data(app_data);
            assert!(host_file_access
                .ensure_read_allowed(&ugc_path, &app_paths)
                .is_err());
            assert!(host_file_access
                .ensure_write_allowed(&ugc_path, &app_paths)
                .is_err());

            register_persisted_user_generated_content_path_grant(&host_file_access, &config)?;

            host_file_access.ensure_read_allowed(&ugc_path, &app_paths)?;
            host_file_access.ensure_write_allowed(ugc_path.join("Prints"), &app_paths)?;
            Ok(())
        }
    }
}

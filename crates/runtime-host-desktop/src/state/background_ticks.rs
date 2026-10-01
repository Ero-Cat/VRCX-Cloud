use std::sync::{Arc, Mutex};

use vrcx_0_application::auth::AuthenticatedSessionProjection;
use vrcx_0_application_core::{
    BackendRuntime, BackendRuntimePhase, BackendRuntimeStatusPublisher,
    BackendRuntimeTelemetryKind, BackgroundCapabilitySession, BackgroundCapabilitySessionIdentity,
    HostSessionRuntime, RemoteMutationGate, RuntimeAuthScope, RuntimeBackgroundJobs,
    RuntimeEventBus,
};
use vrcx_0_application_realtime::RealtimeHostRuntime;
use vrcx_0_persistence::config::ConfigRepository;

mod discord;
mod presence;

pub(in crate::state) use discord::{run_background_discord_tick, DiscordPresenceLabelCache};
pub(in crate::state) use presence::run_background_presence_tick;

pub(in crate::state) const BACKGROUND_PRESENCE_AUTOMATION_JOB: &str =
    "backgroundPresenceAutomation";
pub(in crate::state) const BACKGROUND_DISCORD_PRESENCE_JOB: &str = "backgroundDiscordPresence";
pub(in crate::state) const BACKGROUND_PRESENCE_CADENCE_SECONDS: u64 = 3;
pub(in crate::state) const BACKGROUND_DISCORD_CADENCE_SECONDS: u64 = 3;

pub(in crate::state) struct BackgroundTickContext<'a> {
    pub(in crate::state) db: &'a Arc<vrcx_0_persistence::DatabaseService>,
    pub(in crate::state) web: &'a Arc<vrcx_0_application_core::WebClient>,
    pub(in crate::state) session_slot: &'a Arc<Mutex<AuthenticatedSessionProjection>>,
    pub(in crate::state) realtime_runtime: &'a Arc<RealtimeHostRuntime>,
    pub(in crate::state) host_session: &'a HostSessionRuntime,
    pub(in crate::state) config: &'a ConfigRepository,
    pub(in crate::state) auth_scope: &'a RuntimeAuthScope,
    pub(in crate::state) remote_mutations: &'a Arc<RemoteMutationGate>,
    pub(in crate::state) event_bus: &'a RuntimeEventBus,
    pub(in crate::state) desktop_services: &'a Arc<crate::DesktopRuntimeServices>,
    pub(in crate::state) backend_runtime: &'a BackendRuntime,
    pub(in crate::state) background_jobs: &'a RuntimeBackgroundJobs,
}

pub(in crate::state) fn background_capability_session(
    session_slot: &Arc<Mutex<AuthenticatedSessionProjection>>,
) -> Option<BackgroundCapabilitySession> {
    let slot = session_slot
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    slot.session
        .as_ref()
        .map(|session| BackgroundCapabilitySession {
            auth_scope_generation: session.auth_scope_generation,
            current_user_id: session.user_id.clone(),
            endpoint: session.endpoint.clone(),
            websocket: session.websocket.clone(),
            current_user_snapshot: session.current_user_snapshot.clone(),
        })
}

pub(in crate::state) fn background_capability_session_identity(
    session_slot: &Arc<Mutex<AuthenticatedSessionProjection>>,
) -> Option<BackgroundCapabilitySessionIdentity> {
    let slot = session_slot
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    slot.session
        .as_ref()
        .map(|session| BackgroundCapabilitySessionIdentity {
            auth_scope_generation: session.auth_scope_generation,
            current_user_id: session.user_id.clone(),
            endpoint: session.endpoint.clone(),
            websocket: session.websocket.clone(),
        })
}

pub(in crate::state) fn background_capability_session_matches(
    session_slot: &Arc<Mutex<AuthenticatedSessionProjection>>,
    expected: &BackgroundCapabilitySessionIdentity,
) -> bool {
    let slot = session_slot
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    slot.session
        .as_ref()
        .map(|current| {
            current.auth_scope_generation == expected.auth_scope_generation
                && current.user_id == expected.current_user_id
                && current.endpoint == expected.endpoint
                && current.websocket == expected.websocket
        })
        .unwrap_or(false)
}

pub(in crate::state) fn emit_background_info(
    event_bus: &RuntimeEventBus,
    backend_runtime: &BackendRuntime,
    detail: impl Into<String>,
) {
    emit_background_output(
        event_bus,
        backend_runtime,
        BackendRuntimeTelemetryKind::BackgroundInfo,
        detail,
    );
}

pub(in crate::state) fn emit_background_error(
    event_bus: &RuntimeEventBus,
    backend_runtime: &BackendRuntime,
    detail: impl Into<String>,
) {
    emit_background_output(
        event_bus,
        backend_runtime,
        BackendRuntimeTelemetryKind::BackgroundError,
        detail,
    );
}

pub(in crate::state) fn emit_background_warning(
    event_bus: &RuntimeEventBus,
    backend_runtime: &BackendRuntime,
    detail: impl Into<String>,
) {
    emit_background_output(
        event_bus,
        backend_runtime,
        BackendRuntimeTelemetryKind::BackgroundWarning,
        detail,
    );
}

pub(in crate::state) fn emit_background_info_if_changed(
    event_bus: &RuntimeEventBus,
    backend_runtime: &BackendRuntime,
    last_detail: &mut Option<String>,
    detail: impl Into<String>,
) {
    let detail = detail.into();
    if !remember_background_output_if_changed(last_detail, &detail) {
        return;
    }
    emit_background_info(event_bus, backend_runtime, detail);
}

pub(in crate::state) fn remember_background_output_if_changed(
    last_detail: &mut Option<String>,
    detail: &str,
) -> bool {
    if last_detail.as_deref() == Some(detail) {
        return false;
    }
    *last_detail = Some(detail.into());
    true
}

fn emit_background_output(
    event_bus: &RuntimeEventBus,
    backend_runtime: &BackendRuntime,
    kind: BackendRuntimeTelemetryKind,
    detail: impl Into<String>,
) {
    let snapshot = backend_runtime.snapshot();
    if snapshot.phase != BackendRuntimePhase::Running {
        return;
    }
    BackendRuntimeStatusPublisher::new(backend_runtime.clone(), event_bus.clone())
        .publish_telemetry(kind, detail, snapshot);
}

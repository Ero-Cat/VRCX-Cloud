use vrcx_0_application::auth::{
    invalidate_background_auth_scope, AuthenticatedRuntimeSession, AuthenticatedSessionSnapshot,
    BackgroundAuthRecoveryActions, BackgroundAuthRecoveryContext, BackgroundAuthRecoveryFuture,
};
use vrcx_0_application_activity::notification::{AuthWebhookEvent, AuthWebhookEventKind};
use vrcx_0_application_core::{BackendRuntimeSnapshot, BackendRuntimeTelemetryKind};

use super::RuntimeHostState;

struct RuntimeHostBackgroundAuthRecoveryActions<'a> {
    state: &'a RuntimeHostState,
}

impl BackgroundAuthRecoveryActions for RuntimeHostBackgroundAuthRecoveryActions<'_> {
    fn runtime_snapshot(&self) -> BackendRuntimeSnapshot {
        self.state.backend_runtime.snapshot()
    }

    fn authenticated_session(&self) -> Option<AuthenticatedSessionSnapshot> {
        self.state.authenticated_session_projection().session
    }

    fn record_recovery_started(
        &self,
        context: &BackgroundAuthRecoveryContext,
        snapshot: &BackendRuntimeSnapshot,
    ) {
        self.state.emit_backend_runtime_telemetry_snapshot(
            BackendRuntimeTelemetryKind::AuthRecoveryStarted,
            context.reason.clone(),
            snapshot.clone(),
        );
    }

    fn invalidate_auth_scope(&self) {
        invalidate_background_auth_scope(&self.state.runtime_context.auth_scope);
    }

    fn clear_authenticated_session(&self) {
        self.state.clear_authenticated_session_projection();
    }

    fn set_authenticating(&self) {
        self.state.backend_runtime.set_authenticating();
    }

    fn authenticate_saved_user<'a>(
        &'a self,
        user_id: &'a str,
        endpoint: &'a str,
    ) -> BackgroundAuthRecoveryFuture<'a> {
        Box::pin(async move {
            self.state
                .authenticate_non_interactive_for_saved_user(user_id, endpoint)
                .await
        })
    }

    fn start_authenticated_session(
        &self,
        session: AuthenticatedRuntimeSession,
    ) -> std::result::Result<BackendRuntimeSnapshot, String> {
        self.state
            .start_authenticated_runtime_session(session)
            .map_err(|error| error.to_string())
    }

    fn set_auth_error(&self, reason: &str) -> BackendRuntimeSnapshot {
        self.state.backend_runtime.set_auth_error(reason)
    }

    fn set_auth_interaction_required(&self, reason: &str) -> BackendRuntimeSnapshot {
        self.state
            .backend_runtime
            .set_auth_interaction_required(reason)
    }

    fn clear_invalid_session(&self, user_id: &str, reason: &str) -> BackendRuntimeSnapshot {
        self.state
            .clear_invalid_non_interactive_auth_session(user_id, reason)
    }

    fn record_recovery_failed(
        &self,
        context: &BackgroundAuthRecoveryContext,
        reason: &str,
        snapshot: &BackendRuntimeSnapshot,
    ) {
        self.state.emit_backend_runtime_telemetry_snapshot(
            BackendRuntimeTelemetryKind::AuthRecoveryFailed,
            reason.to_string(),
            snapshot.clone(),
        );
        self.state
            .runtime_context
            .enqueue_auth_webhook(AuthWebhookEvent {
                kind: AuthWebhookEventKind::ReloginFailed,
                user_id: context.user_id.clone(),
                display_name: context.display_name.clone(),
                reason: context.normalized_failure_reason(reason.to_string()),
                mode: context.mode,
                timestamp: context.timestamp.clone(),
            });
    }
}

impl RuntimeHostState {
    pub async fn recover_background_auth_after_failure(
        &self,
        reason: impl Into<String>,
    ) -> BackendRuntimeSnapshot {
        self.background_auth_recovery
            .recover(
                &RuntimeHostBackgroundAuthRecoveryActions { state: self },
                reason.into(),
            )
            .await
    }
}

use futures_util::future::BoxFuture;

use std::sync::atomic::{AtomicBool, Ordering};

use vrcx_0_application_core::{
    BackendRuntimeAuthStatus, BackendRuntimeMode, BackendRuntimePhase, BackendRuntimeSnapshot,
    RuntimeAuthScope,
};
use vrcx_0_core::time::now_iso;
use vrcx_0_core::vrchat_endpoints::normalize_vrchat_api_endpoint;

use super::{AuthenticatedRuntimeSession, AuthenticatedSessionSnapshot, NonInteractiveAuthError};

pub type BackgroundAuthRecoveryFuture<'a> =
    BoxFuture<'a, std::result::Result<AuthenticatedRuntimeSession, NonInteractiveAuthError>>;

pub trait BackgroundAuthRecoveryActions {
    fn runtime_snapshot(&self) -> BackendRuntimeSnapshot;
    fn authenticated_session(&self) -> Option<AuthenticatedSessionSnapshot>;
    fn record_recovery_started(
        &self,
        context: &BackgroundAuthRecoveryContext,
        snapshot: &BackendRuntimeSnapshot,
    );
    fn invalidate_auth_scope(&self);
    fn clear_authenticated_session(&self);
    fn set_authenticating(&self);
    fn authenticate_saved_user<'a>(
        &'a self,
        user_id: &'a str,
        endpoint: &'a str,
    ) -> BackgroundAuthRecoveryFuture<'a>;
    fn start_authenticated_session(
        &self,
        session: AuthenticatedRuntimeSession,
    ) -> std::result::Result<BackendRuntimeSnapshot, String>;
    fn set_auth_error(&self, reason: &str) -> BackendRuntimeSnapshot;
    fn set_auth_interaction_required(&self, reason: &str) -> BackendRuntimeSnapshot;
    fn clear_invalid_session(&self, user_id: &str, reason: &str) -> BackendRuntimeSnapshot;
    fn record_recovery_failed(
        &self,
        context: &BackgroundAuthRecoveryContext,
        reason: &str,
        snapshot: &BackendRuntimeSnapshot,
    );
}

pub struct BackgroundAuthRecoveryOrchestrator {
    running: AtomicBool,
}

impl Default for BackgroundAuthRecoveryOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundAuthRecoveryOrchestrator {
    pub const fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }

    pub async fn recover<A: BackgroundAuthRecoveryActions>(
        &self,
        actions: &A,
        reason: String,
    ) -> BackendRuntimeSnapshot {
        let snapshot = actions.runtime_snapshot();
        if !background_auth_recovery_should_run(&snapshot) {
            return snapshot;
        }
        let Some(_claim) = BackgroundAuthRecoveryClaim::try_acquire(&self.running) else {
            return snapshot;
        };

        let snapshot = actions.runtime_snapshot();
        if !background_auth_recovery_should_run(&snapshot) {
            return snapshot;
        }
        let Some(session) = actions.authenticated_session() else {
            return snapshot;
        };
        let context = BackgroundAuthRecoveryContext::from_session(snapshot.mode, &session, reason);
        actions.record_recovery_started(&context, &snapshot);
        actions.invalidate_auth_scope();
        actions.clear_authenticated_session();
        actions.set_authenticating();

        match actions
            .authenticate_saved_user(&context.user_id, &context.endpoint)
            .await
        {
            Ok(session) => {
                if !context.matches_session(&session) {
                    let reason = "Recovered session does not match dropped background auth scope.";
                    let snapshot = actions.set_auth_error(reason);
                    actions.record_recovery_failed(&context, reason, &snapshot);
                    return snapshot;
                }
                match actions.start_authenticated_session(session) {
                    Ok(snapshot) => snapshot,
                    Err(reason) => {
                        let snapshot = actions.set_auth_error(&reason);
                        actions.record_recovery_failed(&context, &reason, &snapshot);
                        snapshot
                    }
                }
            }
            Err(NonInteractiveAuthError::InteractionRequired(reason)) => {
                let snapshot = actions.set_auth_interaction_required(&reason);
                actions.record_recovery_failed(&context, &reason, &snapshot);
                snapshot
            }
            Err(NonInteractiveAuthError::SessionInvalidated {
                user_id, reason, ..
            }) => {
                let snapshot = actions.clear_invalid_session(&user_id, &reason);
                actions.record_recovery_failed(&context, &reason, &snapshot);
                snapshot
            }
            Err(NonInteractiveAuthError::Failed(reason)) => {
                let snapshot = actions.set_auth_error(&reason);
                actions.record_recovery_failed(&context, &reason, &snapshot);
                snapshot
            }
        }
    }
}

struct BackgroundAuthRecoveryClaim<'a> {
    running: &'a AtomicBool,
}

impl<'a> BackgroundAuthRecoveryClaim<'a> {
    fn try_acquire(running: &'a AtomicBool) -> Option<Self> {
        running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| Self { running })
    }
}

impl Drop for BackgroundAuthRecoveryClaim<'_> {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
    }
}

fn background_auth_recovery_should_run(snapshot: &BackendRuntimeSnapshot) -> bool {
    snapshot.mode == BackendRuntimeMode::Background
        && snapshot.phase == BackendRuntimePhase::Running
        && snapshot.auth_status == BackendRuntimeAuthStatus::Authenticated
        && !snapshot.auth_user_id.trim().is_empty()
}

#[derive(Clone, Debug)]
pub struct BackgroundAuthRecoveryContext {
    pub user_id: String,
    pub display_name: String,
    pub endpoint: String,
    pub reason: String,
    pub mode: BackendRuntimeMode,
    pub timestamp: String,
}

impl BackgroundAuthRecoveryContext {
    pub fn from_session(
        mode: BackendRuntimeMode,
        session: &AuthenticatedSessionSnapshot,
        reason: String,
    ) -> Self {
        Self {
            user_id: session.user_id.trim().to_string(),
            display_name: session.display_name.trim().to_string(),
            endpoint: normalize_vrchat_api_endpoint(Some(&session.endpoint)),
            reason: normalize_recovery_reason(reason),
            mode,
            timestamp: now_iso(),
        }
    }

    pub fn matches_session(&self, session: &AuthenticatedRuntimeSession) -> bool {
        self.user_id == session.user_id.trim()
            && self.endpoint == normalize_vrchat_api_endpoint(Some(&session.endpoint))
    }

    pub fn normalized_failure_reason(&self, reason: String) -> String {
        normalize_recovery_reason(reason)
    }
}

pub fn invalidate_background_auth_scope(auth_scope: &RuntimeAuthScope) {
    auth_scope.set_identity("", "", "");
}

fn normalize_recovery_reason(reason: String) -> String {
    let reason = reason.trim();
    if reason.is_empty() {
        "Background realtime auth failed.".into()
    } else {
        reason.into()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serde_json::json;
    use vrcx_0_application_core::{BackendRuntime, GuiRuntimeMode, RuntimeHostProfile};

    use super::*;

    #[test]
    fn background_auth_recovery_invalidates_the_dropped_login_generation() {
        let auth_scope = RuntimeAuthScope::new();
        let dropped = auth_scope.set("usr_before", "https://api.example.test/api/1");

        invalidate_background_auth_scope(&auth_scope);
        assert!(!auth_scope.snapshot().active);
        let recovered = auth_scope.set("usr_before", "https://api.example.test/api/1");

        assert!(recovered.generation > dropped.generation);
    }

    #[test]
    fn recovery_context_matches_only_dropped_user_and_endpoint() {
        let context = BackgroundAuthRecoveryContext::from_session(
            BackendRuntimeMode::Background,
            &projected_session("usr_before", "https://api.example.test/api/1/"),
            "auth failed".into(),
        );

        assert!(context.matches_session(&session("usr_before", "https://api.example.test/api/1")));
        assert!(!context.matches_session(&session("usr_after", "https://api.example.test/api/1")));
        assert!(!context.matches_session(&session("usr_before", "https://api.other.test/api/1")));
    }

    #[test]
    fn recovery_reason_keeps_the_existing_trim_and_default_contract() {
        let session = projected_session("usr_before", "https://api.example.test/api/1");
        let defaulted = BackgroundAuthRecoveryContext::from_session(
            BackendRuntimeMode::Background,
            &session,
            "   ".into(),
        );
        let trimmed = BackgroundAuthRecoveryContext::from_session(
            BackendRuntimeMode::Background,
            &session,
            "  expired  ".into(),
        );

        assert_eq!(defaulted.reason, "Background realtime auth failed.");
        assert_eq!(trimmed.reason, "expired");
        assert_eq!(
            trimmed.normalized_failure_reason("  failed  ".into()),
            "failed"
        );
    }

    #[tokio::test]
    async fn recovery_orchestrator_preserves_successful_transition_order() {
        let actions =
            FakeRecoveryActions::new(Ok(session("usr_before", "https://api.example.test/api/1")));

        let output = BackgroundAuthRecoveryOrchestrator::new()
            .recover(&actions, "  websocket auth failed  ".into())
            .await;

        assert_eq!(output.phase, BackendRuntimePhase::Running);
        assert_eq!(
            actions.take_log(),
            [
                "started:websocket auth failed",
                "invalidate",
                "clear",
                "authenticating",
                "authenticate:usr_before:https://api.example.test/api/1",
                "start:usr_before",
            ]
        );
    }

    #[tokio::test]
    async fn recovery_orchestrator_preserves_terminal_failure_mapping() {
        let cases = [
            (
                Err(NonInteractiveAuthError::InteractionRequired(
                    "two-factor".into(),
                )),
                "interaction:two-factor",
                "failed:two-factor",
            ),
            (
                Err(NonInteractiveAuthError::SessionInvalidated {
                    user_id: "usr_before".into(),
                    reason: "expired".into(),
                    status_code: Some(401),
                }),
                "invalid:usr_before:expired",
                "failed:expired",
            ),
            (
                Err(NonInteractiveAuthError::Failed("network".into())),
                "error:network",
                "failed:network",
            ),
        ];

        for (outcome, expected_terminal, expected_failure) in cases {
            let actions = FakeRecoveryActions::new(outcome);
            BackgroundAuthRecoveryOrchestrator::new()
                .recover(&actions, "auth failed".into())
                .await;
            let log = actions.take_log();
            assert!(log.iter().any(|entry| entry == expected_terminal));
            assert_eq!(log.last().map(String::as_str), Some(expected_failure));
        }
    }

    #[tokio::test]
    async fn recovery_orchestrator_rejects_a_different_recovered_scope() {
        let actions =
            FakeRecoveryActions::new(Ok(session("usr_other", "https://api.example.test/api/1")));

        let output = BackgroundAuthRecoveryOrchestrator::new()
            .recover(&actions, "auth failed".into())
            .await;

        assert_eq!(output.auth_status, BackendRuntimeAuthStatus::Error);
        assert!(actions.take_log().iter().any(|entry| {
            entry == "error:Recovered session does not match dropped background auth scope."
        }));
    }

    #[tokio::test]
    async fn recovery_orchestrator_ignores_ineligible_runtime_without_side_effects() {
        let mut snapshot = eligible_snapshot();
        snapshot.mode = BackendRuntimeMode::Foreground;
        let actions = FakeRecoveryActions::with_snapshot(
            snapshot.clone(),
            Ok(session("usr_before", "https://api.example.test/api/1")),
        );

        let output = BackgroundAuthRecoveryOrchestrator::new()
            .recover(&actions, "auth failed".into())
            .await;

        assert_eq!(output.mode, snapshot.mode);
        assert!(actions.take_log().is_empty());
    }

    struct FakeRecoveryActions {
        snapshot: BackendRuntimeSnapshot,
        outcome: Mutex<
            Option<std::result::Result<AuthenticatedRuntimeSession, NonInteractiveAuthError>>,
        >,
        log: Mutex<Vec<String>>,
    }

    impl FakeRecoveryActions {
        fn new(
            outcome: std::result::Result<AuthenticatedRuntimeSession, NonInteractiveAuthError>,
        ) -> Self {
            Self::with_snapshot(eligible_snapshot(), outcome)
        }

        fn with_snapshot(
            snapshot: BackendRuntimeSnapshot,
            outcome: std::result::Result<AuthenticatedRuntimeSession, NonInteractiveAuthError>,
        ) -> Self {
            Self {
                snapshot,
                outcome: Mutex::new(Some(outcome)),
                log: Mutex::new(Vec::new()),
            }
        }

        fn record(&self, entry: impl Into<String>) {
            self.log.lock().unwrap().push(entry.into());
        }

        fn take_log(&self) -> Vec<String> {
            std::mem::take(&mut *self.log.lock().unwrap())
        }

        fn terminal_snapshot(
            &self,
            auth_status: BackendRuntimeAuthStatus,
            reason: &str,
        ) -> BackendRuntimeSnapshot {
            let mut snapshot = self.snapshot.clone();
            snapshot.phase = BackendRuntimePhase::Error;
            snapshot.auth_status = auth_status;
            snapshot.last_error = Some(reason.into());
            snapshot
        }
    }

    impl BackgroundAuthRecoveryActions for FakeRecoveryActions {
        fn runtime_snapshot(&self) -> BackendRuntimeSnapshot {
            self.snapshot.clone()
        }

        fn authenticated_session(&self) -> Option<AuthenticatedSessionSnapshot> {
            Some(projected_session(
                "usr_before",
                "https://api.example.test/api/1",
            ))
        }

        fn record_recovery_started(
            &self,
            context: &BackgroundAuthRecoveryContext,
            _snapshot: &BackendRuntimeSnapshot,
        ) {
            self.record(format!("started:{}", context.reason));
        }

        fn invalidate_auth_scope(&self) {
            self.record("invalidate");
        }

        fn clear_authenticated_session(&self) {
            self.record("clear");
        }

        fn set_authenticating(&self) {
            self.record("authenticating");
        }

        fn authenticate_saved_user<'a>(
            &'a self,
            user_id: &'a str,
            endpoint: &'a str,
        ) -> BackgroundAuthRecoveryFuture<'a> {
            self.record(format!("authenticate:{user_id}:{endpoint}"));
            let outcome = self.outcome.lock().unwrap().take().unwrap();
            Box::pin(async move { outcome })
        }

        fn start_authenticated_session(
            &self,
            session: AuthenticatedRuntimeSession,
        ) -> std::result::Result<BackendRuntimeSnapshot, String> {
            self.record(format!("start:{}", session.user_id));
            Ok(self.snapshot.clone())
        }

        fn set_auth_error(&self, reason: &str) -> BackendRuntimeSnapshot {
            self.record(format!("error:{reason}"));
            self.terminal_snapshot(BackendRuntimeAuthStatus::Error, reason)
        }

        fn set_auth_interaction_required(&self, reason: &str) -> BackendRuntimeSnapshot {
            self.record(format!("interaction:{reason}"));
            self.terminal_snapshot(BackendRuntimeAuthStatus::InteractionRequired, reason)
        }

        fn clear_invalid_session(&self, user_id: &str, reason: &str) -> BackendRuntimeSnapshot {
            self.record(format!("invalid:{user_id}:{reason}"));
            self.terminal_snapshot(BackendRuntimeAuthStatus::SignedOut, reason)
        }

        fn record_recovery_failed(
            &self,
            _context: &BackgroundAuthRecoveryContext,
            reason: &str,
            _snapshot: &BackendRuntimeSnapshot,
        ) {
            self.record(format!("failed:{reason}"));
        }
    }

    fn eligible_snapshot() -> BackendRuntimeSnapshot {
        let runtime = BackendRuntime::new(RuntimeHostProfile::Desktop);
        runtime.set_gui_mode(GuiRuntimeMode::Background);
        runtime.set_phase(BackendRuntimePhase::Running);
        runtime.set_auth_success("usr_before", "Pizza")
    }

    fn projected_session(user_id: &str, endpoint: &str) -> AuthenticatedSessionSnapshot {
        AuthenticatedSessionSnapshot {
            auth_scope_generation: 1,
            user_id: user_id.into(),
            display_name: "Pizza".into(),
            endpoint: endpoint.into(),
            websocket: "wss://pipeline.vrchat.cloud".into(),
            current_user_snapshot: json!({
                "id": user_id,
                "displayName": "Pizza"
            })
            .into(),
        }
    }

    fn session(user_id: &str, endpoint: &str) -> AuthenticatedRuntimeSession {
        AuthenticatedRuntimeSession::from_user(
            json!({
                "id": user_id,
                "displayName": "Pizza"
            }),
            endpoint.into(),
            "wss://pipeline.vrchat.cloud".into(),
        )
    }
}

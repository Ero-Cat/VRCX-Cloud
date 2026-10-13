//! Reports VRChat sessions invalidated while serving proxied API commands.
//!
//! VRChat answers a request made with missing or invalidated credentials
//! with HTTP 401 (`Missing Credentials`). The invoke transport wraps that
//! response as a successful command result (`{"ok":true,"result":{...}}`),
//! so without this guard the failure never reaches the runtime auth
//! failure observers and clients keep their `ready` session instead of
//! recovering. The guard mirrors the realtime websocket path: verify the
//! failure belongs to the session that is still current, then emit
//! `runtimeVrchatAuthFailure` so clients end the session and restore it
//! from saved credentials.

use vrcx_0_contracts::vrchat_api::{
    classify_vrchat_auth_failure, VrchatAuthFailureKind, VrchatJsonResponse, VrchatRequest,
    VrchatResponse,
};
use vrcx_0_core::OwnerId;

use crate::{
    RuntimeAuthScope, RuntimeAuthScopeSnapshot, RuntimeEventBus, RuntimeVrchatAuthFailurePayload,
};

#[derive(Clone)]
pub struct WebClientAuthFailureGuard {
    auth_scope: RuntimeAuthScope,
    event_bus: RuntimeEventBus,
}

impl WebClientAuthFailureGuard {
    pub fn new(auth_scope: RuntimeAuthScope, event_bus: RuntimeEventBus) -> Self {
        Self {
            auth_scope,
            event_bus,
        }
    }

    /// Scope snapshot to capture before the request is sent; the failure is
    /// only reported when the scope is still the same when the response
    /// lands, so a mid-flight logout or account switch stays silent.
    pub fn expected_snapshot(&self) -> RuntimeAuthScopeSnapshot {
        self.auth_scope.snapshot()
    }

    pub fn report_auth_failure(
        &self,
        expected: &RuntimeAuthScopeSnapshot,
        request: &VrchatRequest,
        response: &VrchatResponse,
    ) {
        if !is_reportable_auth_failure(response) || is_auth_session_request(request) {
            return;
        }
        let live = self.auth_scope.snapshot();
        if !live.generation_matches(expected) {
            return;
        }
        self.auth_scope.report_vrchat_auth_failure(
            &self.event_bus,
            RuntimeVrchatAuthFailurePayload {
                owner_user_id: OwnerId::new(live.current_user_id.clone()),
                endpoint: live.endpoint.clone(),
                path: failure_path(request),
                reason: VrchatJsonResponse::from(response)
                    .error_message_with_http_status("VRChat API request failed"),
                status_code: response.status,
                auth_scope_generation: live.generation,
                realtime_transport: None,
            },
        );
    }
}

fn is_reportable_auth_failure(response: &VrchatResponse) -> bool {
    // REST 403s are permission problems, not session death — only the
    // websocket path treats a transport-scoped 403 as an auth failure.
    response.status == 401
        && matches!(
            classify_vrchat_auth_failure(response),
            VrchatAuthFailureKind::MissingCredentials | VrchatAuthFailureKind::SessionInvalidated
        )
}

/// Login and session-token requests legitimately answer 401 while
/// credentials are being established; they must not end a live session.
fn is_auth_session_request(request: &VrchatRequest) -> bool {
    request
        .path
        .as_deref()
        .is_some_and(|path| path.trim_start_matches('/').starts_with("auth/"))
}

fn failure_path(request: &VrchatRequest) -> String {
    request
        .path
        .clone()
        .or_else(|| request.url.clone())
        .unwrap_or_else(|| "vrchat-api".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::RuntimeVrchatAuthFailureObserver;
    use crate::{Error, Result, WebClient, WebClientPort};
    use vrcx_0_contracts::external_api::{
        ExternalApiExecuteResponse, ExternalApiScope, ExternalHttpRequestInput,
        ExternalWebExecuteRequest,
    };
    use vrcx_0_contracts::{
        RealtimeAuthTokenFetch, RealtimeConnectionOptions, VrchatScope, WebExecuteRequest,
    };

    const ENDPOINT: &str = "https://api.vrchat.cloud/api/1";

    #[derive(Default)]
    struct FailureRecorder {
        failures: Mutex<Vec<RuntimeVrchatAuthFailurePayload>>,
    }

    impl RuntimeVrchatAuthFailureObserver for FailureRecorder {
        fn runtime_vrchat_auth_failed(&self, failure: &RuntimeVrchatAuthFailurePayload) {
            self.failures
                .lock()
                .expect("failure recorder lock")
                .push(failure.clone());
        }
    }

    struct Fixture {
        auth_scope: RuntimeAuthScope,
        guard: WebClientAuthFailureGuard,
        recorder: Arc<FailureRecorder>,
        event_bus: RuntimeEventBus,
    }

    fn fixture(active: bool) -> Fixture {
        let auth_scope = RuntimeAuthScope::new();
        let event_bus = RuntimeEventBus::new();
        if active {
            auth_scope.set("usr_1", ENDPOINT);
        }
        let recorder = Arc::new(FailureRecorder::default());
        let observer: Arc<dyn RuntimeVrchatAuthFailureObserver> = recorder.clone();
        auth_scope.add_vrchat_auth_failure_observer(observer);
        Fixture {
            guard: WebClientAuthFailureGuard::new(auth_scope.clone(), event_bus.clone()),
            auth_scope,
            recorder,
            event_bus,
        }
    }

    fn missing_credentials_response() -> VrchatResponse {
        VrchatResponse {
            status: 401,
            data: r#"{"error":{"message":"\"Missing Credentials\"","status_code":401}}"#.into(),
        }
    }

    fn user_request() -> VrchatRequest {
        VrchatRequest {
            path: Some("users/usr_1".into()),
            ..VrchatRequest::default()
        }
    }

    fn recorded_failures(fixture: &Fixture) -> Vec<RuntimeVrchatAuthFailurePayload> {
        fixture
            .recorder
            .failures
            .lock()
            .expect("failure recorder lock")
            .clone()
    }

    #[test]
    fn reports_missing_credentials_for_the_current_scope() {
        let fixture = fixture(true);
        let expected = fixture.guard.expected_snapshot();
        fixture.guard.report_auth_failure(
            &expected,
            &user_request(),
            &missing_credentials_response(),
        );

        let failures = recorded_failures(&fixture);
        assert_eq!(failures.len(), 1, "exactly one failure should be reported");
        assert_eq!(failures[0].owner_user_id.as_str(), "usr_1");
        assert_eq!(failures[0].status_code, 401);
        assert_eq!(
            failures[0].path, "users/usr_1",
            "the payload should name the failing request path"
        );
        assert_eq!(failures[0].auth_scope_generation, expected.generation);
        assert_eq!(failures[0].realtime_transport, None);

        let events = fixture.event_bus.take_events_for_test();
        assert!(
            events
                .iter()
                .any(|event| event.name == "runtimeVrchatAuthFailure"),
            "the failure should also be emitted on the runtime event bus"
        );
    }

    #[test]
    fn stays_silent_when_the_scope_changed_mid_flight() {
        let fixture = fixture(true);
        let expected = fixture.guard.expected_snapshot();
        // A mid-flight logout or account switch rotates the scope before
        // the response lands; the stale failure must not be reported.
        fixture.auth_scope.set("usr_2", ENDPOINT);
        fixture.guard.report_auth_failure(
            &expected,
            &user_request(),
            &missing_credentials_response(),
        );

        assert!(
            recorded_failures(&fixture).is_empty(),
            "a stale scope must not report a failure"
        );
    }

    #[test]
    fn stays_silent_without_an_active_scope() {
        let fixture = fixture(false);
        let expected = fixture.guard.expected_snapshot();
        fixture.guard.report_auth_failure(
            &expected,
            &user_request(),
            &missing_credentials_response(),
        );

        assert!(
            recorded_failures(&fixture).is_empty(),
            "logged-out clients must not report session failures"
        );
    }

    #[test]
    fn stays_silent_for_login_and_non_session_failures() {
        let fixture = fixture(true);
        let expected = fixture.guard.expected_snapshot();

        let auth_request = VrchatRequest {
            path: Some("auth/user".into()),
            ..VrchatRequest::default()
        };
        fixture.guard.report_auth_failure(
            &expected,
            &auth_request,
            &missing_credentials_response(),
        );

        let invalid_credentials = VrchatResponse {
            status: 401,
            data: r#"{"error":{"message":"Invalid Username/Email or Password","status_code":401}}"#
                .into(),
        };
        fixture
            .guard
            .report_auth_failure(&expected, &user_request(), &invalid_credentials);

        let forbidden = VrchatResponse {
            status: 403,
            data: r#"{"error":{"message":"Insufficient Permissions","status_code":403}}"#.into(),
        };
        fixture
            .guard
            .report_auth_failure(&expected, &user_request(), &forbidden);

        let ok = VrchatResponse {
            status: 200,
            data: "{}".into(),
        };
        fixture
            .guard
            .report_auth_failure(&expected, &user_request(), &ok);

        assert!(
            recorded_failures(&fixture).is_empty(),
            "auth-session requests and non-session failures must stay silent"
        );
    }

    /// Serves scripted `execute_api` responses; every other port method is
    /// unreachable in these tests.
    struct ScriptedApiPort {
        responses: Mutex<VecDeque<VrchatResponse>>,
    }

    #[async_trait::async_trait]
    impl WebClientPort for ScriptedApiPort {
        fn save_cookies(&self) {}
        fn proxy_url(&self) -> Option<String> {
            None
        }
        async fn fetch_image(&self, _url: &str) -> Result<Vec<u8>> {
            Err(Error::Custom("scripted web client".into()))
        }
        fn realtime_connection_options(&self) -> RealtimeConnectionOptions {
            RealtimeConnectionOptions {
                origin: String::new(),
                proxy_url: None,
            }
        }
        fn clear_cookies(&self) {}
        fn clear_auth_cookies(&self) {}
        fn cookie_diagnostics(&self) -> serde_json::Value {
            serde_json::Value::Null
        }
        fn auth_cookie_value(&self) -> Option<String> {
            None
        }
        fn get_cookies(&self) -> String {
            String::new()
        }
        fn set_cookies(&self, _b64: &str) -> Result<()> {
            Ok(())
        }
        async fn execute(&self, _request: WebExecuteRequest) -> Result<(i32, String)> {
            Err(Error::Custom("scripted web client".into()))
        }
        async fn execute_external(
            &self,
            _request: ExternalWebExecuteRequest,
        ) -> Result<(i32, String)> {
            Err(Error::Custom("scripted web client".into()))
        }
        async fn execute_api(
            &self,
            _input: vrcx_0_contracts::VrchatRequest,
            _scope: VrchatScope,
        ) -> Result<VrchatResponse> {
            Ok(self
                .responses
                .lock()
                .expect("scripted responses lock")
                .pop_front()
                .expect("scripted response exhausted"))
        }
        fn vrchat_config_snapshot(&self, _endpoint: &str) -> Option<VrchatResponse> {
            None
        }
        fn clear_vrchat_config_snapshot(&self) {}
        async fn fetch_realtime_auth_token(
            &self,
            _endpoint: &str,
        ) -> Result<RealtimeAuthTokenFetch> {
            Err(Error::Custom("scripted web client".into()))
        }
        async fn execute_external_api(
            &self,
            _input: ExternalHttpRequestInput,
            _scope: ExternalApiScope,
        ) -> Result<ExternalApiExecuteResponse> {
            Err(Error::Custom("scripted web client".into()))
        }
        async fn execute_external_api_limited(
            &self,
            _input: ExternalHttpRequestInput,
            _scope: ExternalApiScope,
            _max_response_bytes: usize,
        ) -> Result<ExternalApiExecuteResponse> {
            Err(Error::Custom("scripted web client".into()))
        }
    }

    #[tokio::test]
    async fn web_client_facade_reports_proxied_missing_credentials() {
        let auth_scope = RuntimeAuthScope::new();
        auth_scope.set("usr_1", ENDPOINT);
        let event_bus = RuntimeEventBus::new();
        let recorder = Arc::new(FailureRecorder::default());
        let observer: Arc<dyn RuntimeVrchatAuthFailureObserver> = recorder.clone();
        auth_scope.add_vrchat_auth_failure_observer(observer);

        let web = WebClient::new(ScriptedApiPort {
            responses: Mutex::new(VecDeque::from(vec![missing_credentials_response()])),
        });
        web.attach_auth_failure_guard(WebClientAuthFailureGuard::new(
            auth_scope,
            event_bus.clone(),
        ));

        let response = web
            .execute_api(user_request(), VrchatScope::Vrchat)
            .await
            .expect("scripted response");
        assert_eq!(response.status, 401, "the proxied response passes through");

        let failures = recorder.failures.lock().expect("failure recorder lock");
        assert_eq!(
            failures.len(),
            1,
            "the facade reports the 401 through the guard"
        );
        drop(failures);
        assert!(
            event_bus
                .take_events_for_test()
                .iter()
                .any(|event| event.name == "runtimeVrchatAuthFailure"),
            "the failure should be emitted on the runtime event bus"
        );
    }
}

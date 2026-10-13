use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde_json::Value;
use vrcx_0_contracts::external_api::{
    ExternalApiExecuteResponse, ExternalApiScope, ExternalHttpRequestInput,
    ExternalWebExecuteRequest,
};
use vrcx_0_contracts::{
    RealtimeAuthTokenFetch, RealtimeConnectionOptions, VrchatRequest, VrchatResponse, VrchatScope,
    WebExecuteRequest,
};

use crate::web_client_auth_guard::WebClientAuthFailureGuard;
use crate::Result;

#[async_trait]
pub trait WebClientPort: Send + Sync {
    fn save_cookies(&self);
    fn proxy_url(&self) -> Option<String>;
    async fn fetch_image(&self, url: &str) -> Result<Vec<u8>>;
    fn realtime_connection_options(&self) -> RealtimeConnectionOptions;
    fn clear_cookies(&self);
    fn clear_auth_cookies(&self);
    fn cookie_diagnostics(&self) -> Value;
    fn auth_cookie_value(&self) -> Option<String>;
    fn get_cookies(&self) -> String;
    fn set_cookies(&self, b64: &str) -> Result<()>;
    async fn execute(&self, request: WebExecuteRequest) -> Result<(i32, String)>;
    async fn execute_external(&self, request: ExternalWebExecuteRequest) -> Result<(i32, String)>;
    async fn execute_api(&self, input: VrchatRequest, scope: VrchatScope)
        -> Result<VrchatResponse>;
    fn vrchat_config_snapshot(&self, endpoint: &str) -> Option<VrchatResponse>;
    fn clear_vrchat_config_snapshot(&self);
    async fn fetch_realtime_auth_token(&self, endpoint: &str) -> Result<RealtimeAuthTokenFetch>;
    async fn execute_external_api(
        &self,
        input: ExternalHttpRequestInput,
        scope: ExternalApiScope,
    ) -> Result<ExternalApiExecuteResponse>;
    async fn execute_external_api_limited(
        &self,
        input: ExternalHttpRequestInput,
        scope: ExternalApiScope,
        max_response_bytes: usize,
    ) -> Result<ExternalApiExecuteResponse>;
}

pub struct WebClient {
    inner: Arc<dyn WebClientPort>,
    proxy_url: Option<String>,
    auth_failure_guard: RwLock<Option<WebClientAuthFailureGuard>>,
}

impl WebClient {
    pub fn new(inner: impl WebClientPort + 'static) -> Self {
        let proxy_url = inner.proxy_url();
        Self {
            inner: Arc::new(inner),
            proxy_url,
            auth_failure_guard: RwLock::new(None),
        }
    }

    /// Arms the guard that turns proxied VRChat API 401s into
    /// `runtimeVrchatAuthFailure` events. Attach it once the shared auth
    /// scope and event bus exist; without it the client behaves as before.
    pub fn attach_auth_failure_guard(&self, guard: WebClientAuthFailureGuard) {
        match self.auth_failure_guard.write() {
            Ok(mut slot) => *slot = Some(guard),
            Err(error) => tracing::warn!(
                error = %error,
                "failed to attach the web client auth failure guard"
            ),
        }
    }

    fn auth_failure_guard(&self) -> Option<WebClientAuthFailureGuard> {
        self.auth_failure_guard
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn save_cookies(&self) {
        self.inner.save_cookies();
    }

    pub fn proxy_url(&self) -> Option<&str> {
        self.proxy_url.as_deref()
    }

    pub async fn fetch_image(&self, url: &str) -> Result<Vec<u8>> {
        self.inner.fetch_image(url).await
    }

    pub fn realtime_connection_options(&self) -> RealtimeConnectionOptions {
        self.inner.realtime_connection_options()
    }

    pub fn clear_cookies(&self) {
        self.inner.clear_cookies();
    }

    pub fn clear_auth_cookies(&self) {
        self.inner.clear_auth_cookies();
    }

    pub fn cookie_diagnostics(&self) -> Value {
        self.inner.cookie_diagnostics()
    }

    pub fn auth_cookie_value(&self) -> Option<String> {
        self.inner.auth_cookie_value()
    }

    pub fn get_cookies(&self) -> String {
        self.inner.get_cookies()
    }

    pub fn set_cookies(&self, b64: &str) -> Result<()> {
        self.inner.set_cookies(b64)
    }

    pub async fn execute(&self, request: WebExecuteRequest) -> Result<(i32, String)> {
        self.inner.execute(request).await
    }

    pub async fn execute_external(
        &self,
        request: ExternalWebExecuteRequest,
    ) -> Result<(i32, String)> {
        self.inner.execute_external(request).await
    }

    pub async fn execute_api(
        &self,
        input: VrchatRequest,
        scope: VrchatScope,
    ) -> Result<VrchatResponse> {
        let guard = self.auth_failure_guard();
        let expected = guard.as_ref().map(|guard| guard.expected_snapshot());
        let response = self.inner.execute_api(input.clone(), scope).await?;
        if let (Some(guard), Some(expected)) = (guard, expected) {
            guard.report_auth_failure(&expected, &input, &response);
        }
        Ok(response)
    }

    pub fn vrchat_config_snapshot(&self, endpoint: &str) -> Option<VrchatResponse> {
        self.inner.vrchat_config_snapshot(endpoint)
    }

    pub fn clear_vrchat_config_snapshot(&self) {
        self.inner.clear_vrchat_config_snapshot();
    }

    pub async fn fetch_realtime_auth_token(
        &self,
        endpoint: &str,
    ) -> Result<RealtimeAuthTokenFetch> {
        self.inner.fetch_realtime_auth_token(endpoint).await
    }

    pub async fn execute_external_api(
        &self,
        input: ExternalHttpRequestInput,
        scope: ExternalApiScope,
    ) -> Result<ExternalApiExecuteResponse> {
        self.inner.execute_external_api(input, scope).await
    }

    pub async fn execute_external_api_limited(
        &self,
        input: ExternalHttpRequestInput,
        scope: ExternalApiScope,
        max_response_bytes: usize,
    ) -> Result<ExternalApiExecuteResponse> {
        self.inner
            .execute_external_api_limited(input, scope, max_response_bytes)
            .await
    }
}

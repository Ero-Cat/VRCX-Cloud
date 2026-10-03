//! Admin browser gate: an optional second factor in front of the whole
//! web API (`/api/invoke`, `/api/events`, `/api/img`).
//!
//! When an admin password is configured (`[server] admin_password` or
//! `VRCX_CLOUD_ADMIN_PASSWORD`), a browser must unlock once by entering
//! that password at `POST /api/admin/auth`. A successful unlock issues a
//! random token as a permanent HttpOnly cookie; tokens are persisted in
//! `<data_dir>/admin-auth-tokens.json` so they stay valid across server
//! restarts — verify once per browser, permanently. Rotating the admin
//! password invalidates every previously issued token.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Duration;

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub const ADMIN_COOKIE: &str = "vrcxAdminAuth";
/// Ten years: the server-side token itself never expires, and browsers
/// cap persistent-cookie lifetimes anyway.
const COOKIE_MAX_AGE: u64 = 10 * 365 * 24 * 3600;
/// Cheap brute-force damper on the password check.
const WRONG_PASSWORD_DELAY: Duration = Duration::from_millis(400);

#[derive(Debug, Default, Serialize, Deserialize)]
struct TokenFile {
    password_fingerprint: String,
    tokens: HashSet<String>,
}

pub struct AdminAuthState {
    password: String,
    password_fingerprint: String,
    tokens_path: PathBuf,
    tokens: RwLock<HashSet<String>>,
}

impl AdminAuthState {
    /// Load persisted tokens, discarding them when the admin password
    /// has rotated since they were issued.
    pub fn new(password: String, tokens_path: PathBuf) -> Self {
        let password_fingerprint = password_fingerprint(&password);
        let tokens = match load_token_file(&tokens_path) {
            Some(file) if file.password_fingerprint == password_fingerprint => file.tokens,
            _ => HashSet::new(),
        };
        Self {
            password,
            password_fingerprint,
            tokens_path,
            tokens: RwLock::new(tokens),
        }
    }

    pub fn verify_password(&self, candidate: &str) -> bool {
        constant_time_eq(candidate.as_bytes(), self.password.as_bytes())
    }

    /// Issue a fresh token, persist it and hand it to the caller for
    /// the Set-Cookie header.
    pub fn issue_token(&self) -> String {
        // Two v4 UUIDs = 244 random bits; far past any guessing budget.
        let token = format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        );
        if let Ok(mut tokens) = self.tokens.write() {
            tokens.insert(token.clone());
            persist_token_file(&self.tokens_path, &self.password_fingerprint, &tokens);
        }
        token
    }

    pub fn revoke_token(&self, token: &str) {
        if let Ok(mut tokens) = self.tokens.write() {
            if tokens.remove(token) {
                persist_token_file(&self.tokens_path, &self.password_fingerprint, &tokens);
            }
        }
    }

    pub fn is_request_unlocked(&self, headers: &HeaderMap) -> bool {
        let Some(token) = admin_cookie_token(headers) else {
            return false;
        };
        self.tokens
            .read()
            .map(|tokens| tokens.contains(&token))
            .unwrap_or(false)
    }
}

/// Gate a request: `None` when the browser is unlocked (or the gate is
/// disabled), `Some(response)` rejecting it otherwise.
pub fn gate(ctx: &super::WebContext, headers: &HeaderMap) -> Option<Response> {
    let admin = ctx.admin.as_ref()?;
    if admin.is_request_unlocked(headers) {
        None
    } else {
        Some(unauthorized_response())
    }
}

pub fn unauthorized_response() -> Response {
    error_response(
        StatusCode::UNAUTHORIZED,
        "adminAuthRequired",
        "admin password required",
    )
}

fn wrong_password_response() -> Response {
    error_response(
        StatusCode::UNAUTHORIZED,
        "wrongAdminPassword",
        "wrong admin password",
    )
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(json!({
            "ok": false,
            "code": code,
            "message": message,
        })),
    )
        .into_response()
}

fn cookie_header(token: &str, max_age: u64) -> String {
    format!("{ADMIN_COOKIE}={token}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax")
}

#[derive(Deserialize)]
pub struct AdminAuthRequest {
    password: String,
}

/// `GET /api/admin/status` — whether the gate exists and this browser
/// is already unlocked. Always public so the SPA can decide to prompt.
pub async fn status_endpoint(
    State(ctx): State<Arc<super::WebContext>>,
    headers: HeaderMap,
) -> Json<Value> {
    let Some(admin) = ctx.admin.as_ref() else {
        return Json(json!({ "required": false, "unlocked": true }));
    };
    Json(json!({
        "required": true,
        "unlocked": admin.is_request_unlocked(&headers),
    }))
}

/// `POST /api/admin/auth { password }` — verify the admin password and
/// set the permanent unlock cookie on success.
pub async fn auth_endpoint(
    State(ctx): State<Arc<super::WebContext>>,
    Json(request): Json<AdminAuthRequest>,
) -> Response {
    let Some(admin) = ctx.admin.as_ref() else {
        // Gate not configured: nothing to unlock.
        return Json(json!({ "ok": true })).into_response();
    };
    if !admin.verify_password(&request.password) {
        tokio::time::sleep(WRONG_PASSWORD_DELAY).await;
        return wrong_password_response();
    }
    let token = admin.issue_token();
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_header(&token, COOKIE_MAX_AGE))],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

/// `POST /api/admin/logout` — revoke this browser's token and expire
/// the cookie.
pub async fn logout_endpoint(
    State(ctx): State<Arc<super::WebContext>>,
    headers: HeaderMap,
) -> Response {
    if let Some(admin) = ctx.admin.as_ref() {
        if let Some(token) = admin_cookie_token(&headers) {
            admin.revoke_token(&token);
        }
    }
    (
        StatusCode::OK,
        [(header::SET_COOKIE, cookie_header("", 0))],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

fn admin_cookie_token(headers: &HeaderMap) -> Option<String> {
    let header_value = headers.get(header::COOKIE)?.to_str().ok()?;
    header_value.split(';').find_map(|part| {
        let part = part.trim();
        let value = part.strip_prefix(ADMIN_COOKIE)?.strip_prefix('=')?;
        (!value.is_empty()).then(|| value.to_owned())
    })
}

fn password_fingerprint(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= a ^ b;
    }
    diff == 0
}

fn load_token_file(path: &Path) -> Option<TokenFile> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn persist_token_file(path: &Path, fingerprint: &str, tokens: &HashSet<String>) {
    let file = TokenFile {
        password_fingerprint: fingerprint.to_string(),
        tokens: tokens.clone(),
    };
    let content = match serde_json::to_string(&file) {
        Ok(content) => content,
        Err(error) => {
            tracing::warn!(error = %error, "failed to serialize admin auth tokens");
            return;
        }
    };
    if let Err(error) = std::fs::write(path, content) {
        tracing::warn!(
            error = %error,
            path = %path.display(),
            "failed to persist admin auth tokens"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("vrcx-admin-auth-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn path(&self) -> PathBuf {
            self.0.clone()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn headers_with_cookie(cookie: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie).expect("valid cookie header"),
        );
        headers
    }

    #[test]
    fn verifies_the_configured_password_only() {
        let dir = TempDir::new();
        let state = AdminAuthState::new("secret".into(), dir.path().join("tokens.json"));
        assert!(state.verify_password("secret"));
        assert!(!state.verify_password("wrong"));
        assert!(!state.verify_password(""));
    }

    #[test]
    fn issued_token_unlocks_requests_and_survives_restart() {
        let dir = TempDir::new();
        let tokens_path = dir.path().join("tokens.json");
        let state = AdminAuthState::new("secret".into(), tokens_path.clone());
        let token = state.issue_token();

        let headers = headers_with_cookie(&format!("other=1; {ADMIN_COOKIE}={token}"));
        assert!(state.is_request_unlocked(&headers));
        assert!(!state.is_request_unlocked(&HeaderMap::new()));

        // A "restart" reloads the persisted token set.
        let revived = AdminAuthState::new("secret".into(), tokens_path);
        assert!(revived.is_request_unlocked(&headers));
    }

    #[test]
    fn rotating_the_password_invalidates_old_tokens() {
        let dir = TempDir::new();
        let tokens_path = dir.path().join("tokens.json");
        let state = AdminAuthState::new("secret".into(), tokens_path.clone());
        let headers = headers_with_cookie(&format!("{ADMIN_COOKIE}={}", state.issue_token()));

        let rotated = AdminAuthState::new("rotated".into(), tokens_path);
        assert!(!rotated.is_request_unlocked(&headers));
    }

    #[test]
    fn revoked_tokens_stop_unlocking() {
        let dir = TempDir::new();
        let state = AdminAuthState::new("secret".into(), dir.path().join("tokens.json"));
        let token = state.issue_token();
        state.revoke_token(&token);

        let headers = headers_with_cookie(&format!("{ADMIN_COOKIE}={token}"));
        assert!(!state.is_request_unlocked(&headers));
    }

    #[test]
    fn parses_the_admin_cookie_among_others() {
        let dir = TempDir::new();
        let state = AdminAuthState::new("secret".into(), dir.path().join("tokens.json"));
        let token = state.issue_token();

        assert!(state.is_request_unlocked(&headers_with_cookie(&format!("{ADMIN_COOKIE}={token}"))));
        assert!(state.is_request_unlocked(&headers_with_cookie(&format!(
            "a=b; {ADMIN_COOKIE}={token}; c=d"
        ))));
        // Unknown or empty values must not match.
        assert!(!state.is_request_unlocked(&headers_with_cookie("vrcxAdminAuthOther=x")));
        assert!(!state.is_request_unlocked(&headers_with_cookie(&format!("{ADMIN_COOKIE}="))));
    }

    #[test]
    fn constant_time_compares_equal_and_unequal_bytes() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(constant_time_eq(b"", b""));
    }
}

//! Web auth: one password, stateless signed cookie, remembered by the
//! browser.
//!
//! The password comes from `VRCX_CLOUD_WEB_PASSWORD` / `[web] password`.
//! Login issues a persistent cookie carrying a deterministic token
//! derived from the password (SHA-256 over a domain-separated label).
//! Validation recomputes the token, so sessions survive server restarts
//! and the browser never needs to re-enter the password until it clears
//! cookies or the password changes. With no password configured (or
//! `auth_disabled`) the UI is open — trusted-LAN posture only.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tokio::sync::broadcast::Sender;

use super::error::ApiError;

const SESSION_COOKIE: &str = "vrcx_session";
/// Persistent cookie: the browser keeps the login for a year.
const SESSION_TTL: Duration = Duration::from_secs(365 * 24 * 60 * 60);
const LOGIN_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const LOGIN_RATE_LIMIT_MAX: u32 = 10;
const TOKEN_LABEL: &[u8] = b"vrcx-cloud-web-session-v1";

#[derive(Clone)]
pub struct WebAuth {
    /// Deterministic token the valid cookie must carry; None = auth off.
    expected_token: Option<Arc<String>>,
    login_attempts: Arc<Mutex<Vec<Instant>>>,
}

pub struct AuthContext {
    pub auth: WebAuth,
    pub state: Arc<vrcx_0_runtime_host_server::ServerRuntimeHostState>,
    pub events: Sender<(String, Value)>,
    pub registry: super::invoke::CommandRegistry,
}

fn session_token(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(TOKEN_LABEL);
    hasher.update(password.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl WebAuth {
    pub fn new(password: Option<String>) -> Self {
        Self {
            expected_token: password
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(session_token)
                .map(Arc::new),
            login_attempts: Arc::default(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.expected_token.is_some()
    }

    fn token_matches(&self, candidate: &str) -> bool {
        let Some(expected) = self.expected_token.as_deref() else {
            return false;
        };
        // Both sides are fixed-length hex digests; compare all bytes.
        let a = expected.as_bytes();
        let b = candidate.as_bytes();
        let mut diff = (a.len() ^ b.len()) as u8;
        for (x, y) in a.iter().zip(b.iter().chain(std::iter::repeat(&0))) {
            diff |= x ^ y;
        }
        diff == 0 && a.len() == b.len()
    }

    fn throttle(&self) -> Result<(), ApiError> {
        let now = Instant::now();
        let mut attempts = self
            .login_attempts
            .lock()
            .map_err(|_| ApiError::Message("login limiter lock poisoned".into()))?;
        attempts.retain(|at| now.duration_since(*at) < LOGIN_RATE_LIMIT_WINDOW);
        if attempts.len() >= LOGIN_RATE_LIMIT_MAX as usize {
            return Err(ApiError::BadRequest(
                "too many login attempts; wait a minute".into(),
            ));
        }
        attempts.push(now);
        Ok(())
    }

    pub fn login(&self, password: &str) -> Result<String, ApiError> {
        self.throttle()?;
        let token = session_token(password);
        if !self.token_matches(&token) {
            return Err(ApiError::BadRequest("incorrect password".into()));
        }
        Ok(token)
    }

    pub fn validate(&self, token: &str) -> bool {
        self.token_matches(token)
    }

    pub fn session_from_headers(&self, headers: &HeaderMap) -> bool {
        if !self.is_enabled() {
            return true;
        }
        let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) else {
            return false;
        };
        cookie
            .split(';')
            .find_map(|part| {
                let (name, value) = part.trim().split_once('=')?;
                (name == SESSION_COOKIE && self.validate(value)).then_some(())
            })
            .is_some()
    }
}

pub async fn auth_status(State(ctx): State<Arc<AuthContext>>) -> Response {
    let enabled = ctx.auth.is_enabled();
    Json(json!({ "authEnabled": enabled })).into_response()
}

/// Whether the caller holds a valid session (always true when auth is off).
pub async fn auth_session(State(ctx): State<Arc<AuthContext>>, headers: HeaderMap) -> Response {
    let valid = ctx.auth.session_from_headers(&headers);
    Json(json!({ "valid": valid })).into_response()
}

pub async fn auth_login(State(ctx): State<Arc<AuthContext>>, Json(body): Json<Value>) -> Response {
    let password = body
        .get("password")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match ctx.auth.login(password) {
        Ok(token) => {
            let cookie = format!(
                "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
                SESSION_TTL.as_secs()
            );
            (
                StatusCode::OK,
                [(header::SET_COOKIE, cookie)],
                Json(json!({ "ok": true })),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

pub async fn auth_logout(State(_ctx): State<Arc<AuthContext>>, _headers: HeaderMap) -> Response {
    // Stateless tokens can't be revoked server-side; clearing the cookie
    // is the logout. Rotating the password invalidates every issued token.
    let clear = format!("{SESSION_COOKIE}=; Path=/; HttpOnly; Max-Age=0");
    (
        StatusCode::OK,
        [(header::SET_COOKIE, clear)],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

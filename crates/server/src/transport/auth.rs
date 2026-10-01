//! Web session auth: one password, in-memory sessions, cookie based.
//!
//! The password comes from `VRCX_CLOUD_WEB_PASSWORD` / `[web] password`.
//! With no password configured (or `auth_disabled`) the UI is open — a
//! trusted-LAN posture only; remote access should sit behind a VPN.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use tokio::sync::broadcast::Sender;

use super::error::ApiError;

const SESSION_COOKIE: &str = "vrcx_session";
const SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const LOGIN_RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const LOGIN_RATE_LIMIT_MAX: u32 = 10;

#[derive(Clone)]
pub struct WebAuth {
    password: Option<Arc<String>>,
    sessions: Arc<Mutex<HashMap<String, Instant>>>,
    login_attempts: Arc<Mutex<Vec<Instant>>>,
}

pub struct AuthContext {
    pub auth: WebAuth,
    pub state: Arc<vrcx_0_runtime_host_server::ServerRuntimeHostState>,
    pub events: Sender<(String, Value)>,
    pub registry: super::invoke::CommandRegistry,
}

impl WebAuth {
    pub fn new(password: Option<String>) -> Self {
        Self {
            password: password.map(Arc::new),
            sessions: Arc::default(),
            login_attempts: Arc::default(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.password.is_some()
    }

    /// Constant-time-ish comparison to avoid trivial timing oracles.
    fn password_matches(&self, candidate: &str) -> bool {
        let Some(expected) = self.password.as_deref() else {
            return false;
        };
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
        if !self.password_matches(password) {
            return Err(ApiError::BadRequest("incorrect password".into()));
        }
        let token = format!("{}", uuid::Uuid::new_v4().simple());
        self.sessions
            .lock()
            .map_err(|_| ApiError::Message("session store lock poisoned".into()))?
            .insert(token.clone(), Instant::now() + SESSION_TTL);
        Ok(token)
    }

    pub fn logout(&self, token: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(token);
        }
    }

    pub fn validate(&self, token: &str) -> bool {
        let Ok(mut sessions) = self.sessions.lock() else {
            return false;
        };
        match sessions.get(token) {
            Some(expiry) if *expiry > Instant::now() => true,
            Some(_) => {
                sessions.remove(token);
                false
            }
            None => false,
        }
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

pub async fn auth_logout(State(ctx): State<Arc<AuthContext>>, headers: HeaderMap) -> Response {
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        if let Some(token) = cookie.split(';').find_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            (name == SESSION_COOKIE).then_some(value.to_string())
        }) {
            ctx.auth.logout(&token);
        }
    }
    let clear = format!("{SESSION_COOKIE}=; Path=/; HttpOnly; Max-Age=0");
    (
        StatusCode::OK,
        [(header::SET_COOKIE, clear)],
        Json(json!({ "ok": true })),
    )
        .into_response()
}

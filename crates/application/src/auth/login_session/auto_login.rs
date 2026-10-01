use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::auth::AuthCredentialStore;
use serde::{Deserialize, Serialize};
use vrcx_0_core::vrchat_endpoints::VRCHAT_API_DEFAULT_ENDPOINT;

use crate::auth::{
    saved_snapshot, AuthSessionCookies, SavedAuthAutoLoginStatus, SavedAuthSnapshot,
};

use super::runtime::{
    apply_login_failure_cleanup, clear_auth_cookies_and_save, LoginAttemptPolicy,
    LoginSessionOperation,
};
use super::service::{start_cookie_restore, start_saved_credential_login};
use super::types::{LoginApi, LoginFailureKind, LoginSessionState};

const AUTO_LOGIN_WINDOW: Duration = Duration::from_secs(60 * 60);
const AUTO_LOGIN_MAX_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AutoLoginStartInput {
    #[serde(default)]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum AutoLoginTerminalOutcome {
    Throttled { snapshot: Box<SavedAuthSnapshot> },
    Expired { snapshot: Box<SavedAuthSnapshot> },
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(untagged)]
pub enum AutoLoginOutcome {
    Session(LoginSessionState),
    Terminal(AutoLoginTerminalOutcome),
}

pub(super) enum AutoLoginDrive {
    Install(LoginSessionState),
    Done(Box<AutoLoginOutcome>),
}

pub(super) struct AutoLoginThrottle {
    attempts_by_key: Mutex<HashMap<String, Vec<Instant>>>,
}

impl AutoLoginThrottle {
    pub(super) fn new() -> Self {
        Self {
            attempts_by_key: Mutex::new(HashMap::new()),
        }
    }

    fn normalize_key(user_id: &str) -> String {
        let trimmed = user_id.trim();
        if trimmed.is_empty() {
            "__global__".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn prune(bucket: &mut Vec<Instant>, now: Instant) {
        bucket.retain(|attempt| now.saturating_duration_since(*attempt) < AUTO_LOGIN_WINDOW);
    }

    fn attempt_count(&self, user_id: &str, now: Instant) -> usize {
        let key = Self::normalize_key(user_id);
        let mut attempts = self.attempts_by_key.lock().unwrap();
        let bucket = attempts.entry(key).or_default();
        Self::prune(bucket, now);
        bucket.len()
    }

    fn can_attempt(&self, user_id: &str, now: Instant) -> bool {
        self.attempt_count(user_id, now) < AUTO_LOGIN_MAX_ATTEMPTS
    }

    fn record_attempt(&self, user_id: &str, now: Instant) -> usize {
        let key = Self::normalize_key(user_id);
        let mut attempts = self.attempts_by_key.lock().unwrap();
        let bucket = attempts.entry(key).or_default();
        Self::prune(bucket, now);
        bucket.push(now);
        bucket.len()
    }

    pub(super) fn reset_all(&self) {
        self.attempts_by_key.lock().unwrap().clear();
    }
}

impl Default for AutoLoginThrottle {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) async fn drive_auto_login(
    api: &dyn LoginApi,
    config: &dyn AuthCredentialStore,
    cookies: &dyn AuthSessionCookies,
    throttle: &AutoLoginThrottle,
    operation: &LoginSessionOperation,
    input: AutoLoginStartInput,
) -> vrcx_0_application_core::Result<AutoLoginDrive> {
    let user_id = input.user_id.trim().to_string();

    let can_attempt =
        operation.run_if_current(|| Ok(throttle.can_attempt(&user_id, Instant::now())))?;
    if !can_attempt {
        let cleanup_result = operation.run_if_current(|| {
            Ok(apply_failure_cleanup(
                cookies,
                config,
                &user_id,
                LoginFailureKind::SessionInvalidated,
            ))
        })?;
        let outcome = match cleanup_result {
            Ok(snapshot) => AutoLoginOutcome::Terminal(AutoLoginTerminalOutcome::Throttled {
                snapshot: Box::new(snapshot),
            }),
            Err(error) => failure_outcome(
                operation,
                config,
                error.to_string(),
                LoginFailureKind::Other,
            )?,
        };
        return Ok(AutoLoginDrive::Done(Box::new(outcome)));
    }
    operation.run_if_current(|| {
        throttle.record_attempt(&user_id, Instant::now());
        Ok(())
    })?;

    let cookie_state = start_cookie_restore(api, VRCHAT_API_DEFAULT_ENDPOINT, &user_id).await;
    operation.ensure_current()?;

    let is_missing_credentials = matches!(
        cookie_state,
        LoginSessionState::Failed {
            kind: LoginFailureKind::MissingCredentials,
            ..
        }
    );

    if !is_missing_credentials {
        return Ok(AutoLoginDrive::Install(cookie_state));
    }

    operation.run_if_current(|| {
        clear_auth_cookies_and_save(cookies);
        Ok(())
    })?;

    let probe_snapshot = operation.run_if_current(|| saved_snapshot(config))?;
    let fallback_available = probe_snapshot.auto_login_status
        == SavedAuthAutoLoginStatus::Available
        && probe_snapshot
            .saved_credentials_list
            .iter()
            .any(|credential| credential.user.id == user_id);
    if !fallback_available {
        return Ok(AutoLoginDrive::Done(Box::new(AutoLoginOutcome::Terminal(
            AutoLoginTerminalOutcome::Expired {
                snapshot: Box::new(probe_snapshot),
            },
        ))));
    }

    let saved_state = start_saved_credential_login(
        api,
        config,
        cookies,
        VRCHAT_API_DEFAULT_ENDPOINT.to_string(),
        user_id.clone(),
    )
    .await;
    operation.ensure_current()?;
    Ok(AutoLoginDrive::Install(saved_state))
}

fn failure_outcome(
    operation: &LoginSessionOperation,
    config: &dyn AuthCredentialStore,
    reason: String,
    kind: LoginFailureKind,
) -> vrcx_0_application_core::Result<AutoLoginOutcome> {
    let snapshot = operation.run_if_current(|| saved_snapshot(config))?;
    Ok(AutoLoginOutcome::Session(LoginSessionState::Failed {
        reason,
        kind,
        snapshot: Some(Box::new(snapshot)),
    }))
}

fn apply_failure_cleanup(
    cookies: &dyn AuthSessionCookies,
    config: &dyn AuthCredentialStore,
    user_id: &str,
    kind: LoginFailureKind,
) -> vrcx_0_application_core::Result<SavedAuthSnapshot> {
    apply_login_failure_cleanup(
        cookies,
        config,
        &LoginAttemptPolicy::SavedCredential {
            user_id: user_id.to_string(),
        },
        kind,
    )
}

#[cfg(test)]
mod tests;

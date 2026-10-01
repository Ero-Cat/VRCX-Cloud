//! Server-side command implementations for `/api/invoke`.
//!
//! Names and kwargs mirror the desktop bindings (`app__snake_case` with
//! camelCase argument keys) so the same frontend call surface works over
//! HTTP. Desktop-only commands are intentionally absent and resolve to
//! `unsupportedOnWeb`. Blocking SQLite work goes through
//! `spawn_blocking`, matching the desktop wrapper behaviour.

pub mod auth;
pub mod core;
pub mod local;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use serde::Serialize;

pub(crate) fn ok<T: Serialize>(value: T) -> Result<serde_json::Value, ApiError> {
    serde_json::to_value(value)
        .map_err(|error| ApiError::Message(format!("failed to serialize response: {error}")))
}

/// Blocking SQLite work must leave the async workers, exactly like the
/// desktop's `tauri::async_runtime::spawn_blocking`.
pub(crate) async fn run_blocking<T, F>(label: &str, operation: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, vrcx_0_composition::Error> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| ApiError::Message(format!("{label} task failed: {error}")))?
        .map_err(ApiError::from)
}

pub fn build_registry() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    auth::register(&mut registry);
    core::register(&mut registry);
    local::register(&mut registry);
    registry
}

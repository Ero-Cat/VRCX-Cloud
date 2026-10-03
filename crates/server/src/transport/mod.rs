//! Web transport: command dispatch, realtime events, image cache
//! and static SPA serving.

pub mod admin_auth;
pub mod error;
pub mod events;
pub mod img;
pub mod invoke;
pub mod public;
pub mod static_files;

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::broadcast::Sender;
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

/// Shared state for the web transport endpoints: the runtime, the
/// realtime broadcast channel, the command registry installed as the
/// runtime event sink, and the optional admin browser gate.
pub struct WebContext {
    pub state: Arc<ServerRuntimeHostState>,
    pub events: Sender<(String, Value)>,
    pub registry: invoke::CommandRegistry,
    /// When configured, `/api/invoke`, `/api/events` and `/api/img`
    /// require the permanent admin-unlock cookie.
    pub admin: Option<Arc<admin_auth::AdminAuthState>>,
}

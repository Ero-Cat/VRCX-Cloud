//! Command dispatch: `POST /api/invoke { cmd, args }`.
//!
//! Handlers are registered per command name (the same `app__snake_case`
//! names the desktop bindings use) as typed async functions over the
//! server runtime state. Unregistered names resolve to a structured
//! `unsupportedOnWeb` error so the frontend can degrade gracefully.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::error::ApiError;

pub type CommandFuture = Pin<Box<dyn Future<Output = Result<Value, ApiError>> + Send>>;
pub type CommandHandler =
    Arc<dyn Fn(Arc<ServerRuntimeHostState>, Value) -> CommandFuture + Send + Sync>;

#[derive(Default)]
pub struct CommandRegistry {
    handlers: HashMap<&'static str, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<F, Fut>(&mut self, name: &'static str, handler: F)
    where
        F: Fn(Arc<ServerRuntimeHostState>, Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Value, ApiError>> + Send + 'static,
    {
        self.handlers.insert(
            name,
            Arc::new(move |state, args| Box::pin(handler(state, args))),
        );
    }

    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    pub fn dispatch(
        &self,
        state: Arc<ServerRuntimeHostState>,
        name: &str,
        args: Value,
    ) -> Option<CommandFuture> {
        self.handlers.get(name).map(|handler| handler(state, args))
    }
}

#[derive(Deserialize)]
pub struct InvokeRequest {
    pub cmd: String,
    #[serde(default)]
    pub args: Value,
}

pub async fn invoke_endpoint(
    State(ctx): State<Arc<super::auth::AuthContext>>,
    headers: HeaderMap,
    Json(request): Json<InvokeRequest>,
) -> Response {
    if !ctx.auth.session_from_headers(&headers) {
        return ApiError::Unauthorized.into_response();
    }
    let Some(future) = ctx
        .registry
        .dispatch(Arc::clone(&ctx.state), &request.cmd, request.args)
    else {
        tracing::warn!(cmd = %request.cmd, "invoke: unsupported on web");
        return ApiError::Unsupported(request.cmd).into_response();
    };
    match future.await {
        Ok(value) => {
            tracing::debug!(cmd = %request.cmd, "invoke: ok");
            Json(json!({ "ok": true, "result": value })).into_response()
        }
        Err(error) => {
            tracing::warn!(cmd = %request.cmd, code = error.code(), "invoke: failed");
            error.into_response()
        }
    }
}

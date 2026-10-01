//! VRChat authentication commands (login session / 2FA / saved
//! credentials) — the web login flow drives these directly.

use std::sync::Arc;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_application::auth::{
    AutoLoginStartInput, LoginSessionCancelInput, LoginSessionEnd, LoginSessionRespondInput,
    LoginSessionStartInput,
};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedCredentialDeleteInput {
    #[serde(default)]
    user_id: String,
}

use super::ok;

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register(
        "app__vrchat_auth_saved_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move { ok(state.saved_auth_snapshot()?) },
    );
    registry.register(
        "app__vrchat_auth_session_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LoginSessionStartInput = arg(&args, "input")?;
            ok(state.start_login_session(input).await)
        },
    );
    registry.register(
        "app__vrchat_auth_auto_login_start",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: AutoLoginStartInput = arg(&args, "input")?;
            ok(state.start_auto_login(input).await?)
        },
    );
    registry.register(
        "app__vrchat_auth_session_respond",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LoginSessionRespondInput = arg(&args, "input")?;
            ok(state.respond_login_session(input).await)
        },
    );
    registry.register(
        "app__vrchat_auth_session_cancel",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LoginSessionCancelInput = arg(&args, "input")?;
            ok(state.cancel_login_session(input).await)
        },
    );
    registry.register(
        "app__vrchat_auth_session_end",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: LoginSessionEnd = arg(&args, "input")?;
            ok(state.end_login_session(input).await?)
        },
    );
    registry.register(
        "app__vrchat_auth_saved_credential_delete",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: SavedCredentialDeleteInput = arg(&args, "input")?;
            ok(state.delete_saved_credential(input.user_id)?)
        },
    );
    registry.register(
        "app__vrchat_auth_config_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.vrchat_config().get().await?)
        },
    );
    registry.register(
        "app__vrchat_auth_config_refresh",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.vrchat_config().refresh().await?)
        },
    );
    registry.register(
        "app__vrchat_auth_current_user_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.vrchat_remote().current_user().await?)
        },
    );
    registry.register(
        "app__vrchat_auth_visits_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.vrchat_remote().visits().await?)
        },
    );
    registry.register(
        "app__backend_runtime_combined_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.backend_runtime_combined_snapshot())
        },
    );
}

//! Core commands: host capabilities, startup bootstrap, config KV,
//! frontend storage KV and remote-sync status for the web panel.

use std::sync::Arc;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_runtime_host_server::local_data::ConfigWriteEntry;
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register("app__get_host_capabilities", |_state, _args| async move {
        ok(vrcx_0_runtime_host_server::server_host_capabilities())
    });
    registry.register(
        "app__startup_bootstrap_snapshot_get",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("startup bootstrap", move || {
                state.startup_bootstrap_snapshot()
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__config_list_values",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            run_blocking("config list", move || {
                state
                    .local_data()
                    .config_list_values()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__config_set_values",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let entries: Vec<ConfigWriteEntry> = arg(&args, "entries")?;
            run_blocking("config set", move || state.config_set_values(entries)).await?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "app__config_remove_value",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let key: String = arg(&args, "key")?;
            run_blocking("config remove", move || state.config_remove_value(key))
                .await
                .and_then(ok)
        },
    );

    registry.register(
        "storage__set",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let key: String = arg(&args, "key")?;
            let value: String = arg(&args, "value")?;
            state.storage_set(key, value);
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "storage__flush",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            state.storage_flush()?;
            ok(serde_json::json!(null))
        },
    );
    registry.register(
        "storage__remove",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let key: String = arg(&args, "key")?;
            ok(state.storage_remove(&key))
        },
    );
    registry.register(
        "storage__get_all",
        |state: Arc<ServerRuntimeHostState>, _args| async move { ok(state.storage_snapshot()) },
    );

    registry.register(
        "sync__status",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.remote_sync().status().await)
        },
    );
    registry.register(
        "sync__get_connection",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.remote_sync().connection_fields())
        },
    );
    registry.register(
        "sync__trigger_now",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state.remote_sync().run_cycle_now().await?)
        },
    );
}

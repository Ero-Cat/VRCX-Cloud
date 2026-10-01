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

    // Legacy VRCX (desktop) migration never applies on a server: there is
    // no local VRCX install to import from.
    registry.register(
        "app__get_legacy_vrcx_migration_status",
        |_state, _args| async move {
            ok(serde_json::json!({
                "detected": false,
                "available": false,
                "version": null,
                "dbPath": null,
                "configPath": null,
                "reason": null
            }))
        },
    );
    registry.register("app__is_legacy_vrcx_running", |_state, _args| async move {
        ok(false)
    });

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
    registry.register(
        "sync__bootstrap_progress",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            ok(state
                .remote_sync()
                .current()
                .map(|engine| engine.bootstrap_progress())
                .unwrap_or_default())
        },
    );
    registry.register(
        "sync__test_connection",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: vrcx_0_contracts::SyncConnectionInput = arg(&args, "connection")?;
            let result = state
                .remote_sync()
                .test_connection(
                    &input.host,
                    input.port,
                    &input.user,
                    input.password.as_deref(),
                    &input.database,
                    input.tls_verify,
                    input.allow_plaintext,
                )
                .await;
            ok(result)
        },
    );
    registry.register(
        "sync__configure",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let enabled: Option<bool> = serde_json::from_value(
                args.get("enabled")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )
            .map_err(|e| ApiError::BadRequest(format!("invalid `enabled`: {e}")))?;
            let interval_sec: Option<i64> = serde_json::from_value(
                args.get("intervalSec")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )
            .map_err(|e| ApiError::BadRequest(format!("invalid `intervalSec`: {e}")))?;
            let connection: Option<vrcx_0_contracts::SyncConnectionInput> = serde_json::from_value(
                args.get("connection")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            )
            .map_err(|e| ApiError::BadRequest(format!("invalid `connection`: {e}")))?;

            let mut entries: Vec<vrcx_0_runtime_host_server::local_data::ConfigWriteEntry> =
                Vec::new();
            let mut entry = |key: &str, value: serde_json::Value| {
                entries.push(vrcx_0_runtime_host_server::local_data::ConfigWriteEntry {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            };
            if let Some(enabled) = enabled {
                entry("remoteSync.enabled", serde_json::json!(enabled));
            }
            if let Some(fields) = connection {
                entry("remoteSync.host", serde_json::json!(fields.host.trim()));
                if (1..65536).contains(&fields.port) {
                    entry("remoteSync.port", serde_json::json!(fields.port));
                }
                entry("remoteSync.user", serde_json::json!(fields.user.trim()));
                if let Some(password) = fields.password {
                    if !password.trim().is_empty() {
                        entry("remoteSync.password", serde_json::json!(password.trim()));
                    }
                }
                entry(
                    "remoteSync.database",
                    serde_json::json!(fields.database.trim()),
                );
                entry("remoteSync.tlsVerify", serde_json::json!(fields.tls_verify));
                entry(
                    "remoteSync.allowPlaintext",
                    serde_json::json!(fields.allow_plaintext),
                );
            }
            if let Some(interval) = interval_sec {
                entry("remoteSync.intervalSec", serde_json::json!(interval.max(5)));
            }
            if !entries.is_empty() {
                let state_for_write = Arc::clone(&state);
                run_blocking("sync configure", move || {
                    state_for_write.config_set_values(entries)
                })
                .await?;
            }
            let host = state.remote_sync();
            let engine_running = host.current().is_some();
            if enabled.is_some() || engine_running {
                host.restart_from_config().await?;
            }
            ok(host.status().await)
        },
    );
}

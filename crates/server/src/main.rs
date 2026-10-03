//! Self-hosted VRCX web server entry point.
//!
//! Boots one full VRCX runtime (VRChat realtime session + persistence +
//! remote sync) under the Server host profile and serves the web API.
//! The web transport itself lands with the axum command layer; this
//! entry point establishes the runtime lifecycle: config load, sync
//! seeding, non-interactive VRChat auth, health endpoint and graceful
//! shutdown.

mod commands;
mod config;
mod realtime_gate;
mod realtime_supervisor;
mod transport;

use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use vrcx_0_application_core::{
    recommended_tokio_max_blocking_threads, recommended_tokio_worker_threads, RuntimeTask,
    RuntimeTaskExecutor, RuntimeTaskHandle,
};
use vrcx_0_application_sync::{
    CONFIG_ALLOW_PLAINTEXT, CONFIG_DATABASE, CONFIG_ENABLED, CONFIG_HOST, CONFIG_INTERVAL_SEC,
    CONFIG_PASSWORD, CONFIG_PORT, CONFIG_TLS_VERIFY, CONFIG_USER, DEFAULT_INTERVAL_SEC,
};
use vrcx_0_persistence::config::{set_bool, set_string};
use vrcx_0_platform::app_paths::{AppDataDirResolution, AppDataDirSource};
use vrcx_0_runtime_host_server::{ServerRuntimeHostOptions, ServerRuntimeHostState};

use config::ServerConfig;

fn main() -> ExitCode {
    build_adaptive_tokio_runtime().block_on(async_main())
}

fn build_adaptive_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(recommended_tokio_worker_threads())
        .max_blocking_threads(recommended_tokio_max_blocking_threads())
        .thread_name("vrcx-0-server")
        .enable_all()
        .build()
        .expect("failed to build server async runtime")
}

async fn async_main() -> ExitCode {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    init_tracing();

    let config = match ServerConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load server config: {error}");
            return ExitCode::from(1);
        }
    };

    let app_data_dir = resolve_server_app_data_dir(&config.data_dir);
    tracing::info!(
        data_dir = %app_data_dir.current_dir.display(),
        listen_addr = %config.listen_addr,
        "starting VRCX server runtime"
    );

    // Desktop-activity gating: while the user's desktop VRCX-0 is
    // actively syncing, the server keeps its own VRChat websocket off.
    let pause_gate = config
        .realtime_auto_gate
        .then(realtime_gate::PauseGate::new);
    let transport_wrapper = pause_gate.as_ref().map(|gate| {
        let gate = gate.clone();
        Arc::new(
            move |transport: Arc<dyn vrcx_0_application_realtime::RealtimeTransport>| {
                Arc::new(realtime_gate::GatedRealtimeTransport::new(
                    transport,
                    gate.clone(),
                )) as Arc<dyn vrcx_0_application_realtime::RealtimeTransport>
            },
        ) as vrcx_0_composition::RealtimeTransportWrapper
    });
    let state = match ServerRuntimeHostState::new(ServerRuntimeHostOptions {
        realtime_origin: String::new(),
        launched_from_autostart: false,
        app_data_dir,
        app_version: product_app_version(),
        database_maintenance_cache_dir: None,
        task_executor: Arc::new(TokioRuntimeTaskExecutor),
        realtime_transport_wrapper: transport_wrapper,
    }) {
        Ok(state) => Arc::new(state),
        Err(error) => {
            tracing::error!(error = %error, "server runtime construction failed");
            return ExitCode::from(1);
        }
    };

    if let Err(error) = seed_sync_settings(&state, &config.sync) {
        tracing::warn!(error = %error, "failed to seed remote sync settings");
    }

    // Web transport context: realtime broadcast and the command registry
    // installed as the runtime event sink. The admin browser gate is
    // active only when an admin password is configured.
    let admin_auth = config.admin_password.clone().map(|password| {
        let tokens_path = config.data_dir.join("admin-auth-tokens.json");
        if let Err(error) = std::fs::create_dir_all(&config.data_dir) {
            tracing::warn!(error = %error, "failed to create admin auth token dir");
        }
        tracing::info!("admin browser gate enabled (web API requires a one-time browser unlock)");
        Arc::new(transport::admin_auth::AdminAuthState::new(
            password,
            tokens_path,
        ))
    });
    let (event_tx, _) = tokio::sync::broadcast::channel::<(String, Value)>(1024);
    state.set_runtime_event_sink(transport::events::WebEventSink::new(event_tx.clone()));
    let ctx = Arc::new(transport::WebContext {
        state: Arc::clone(&state),
        events: event_tx,
        registry: commands::build_registry(),
        admin: admin_auth,
    });

    if let Err(error) = state.start_headless_backend_runtime().await {
        let reason = error.to_string();
        let awaiting_web_login = reason.contains("No saved account is available")
            || reason.contains("Saved credentials are not available");
        if awaiting_web_login {
            // First boot (or logged out): data services and remote sync are
            // already running; the user completes VRChat login from the web.
            tracing::info!("no saved VRChat account yet - awaiting login from the web UI");
        } else {
            tracing::error!(error = %reason, "backend runtime startup failed");
            shutdown(&state, "startup-failed");
            return ExitCode::from(1);
        }
    } else {
        tracing::info!("backend runtime started from the saved VRChat session");
    }

    // Dual realtime sessions (desktop + server) can double-record feed
    // rows; users running the desktop as primary can turn the server's
    // own feed logging off.
    // Activity aggregates are derived caches the desktop rebuilds when
    // its game log ingests; on the server rows arrive via sync, so
    // re-derive on a cadence to keep the activity pages current.
    spawn_activity_refresh(Arc::clone(&state));

    if let Some(gate) = pause_gate.clone() {
        realtime_supervisor::spawn(Arc::clone(&state), gate);
        tracing::info!(
            "realtime mode: auto (server pauses its VRChat session while a desktop device is active)"
        );
    }

    if config.feed_logging == Some(false) {
        if let Err(error) = state.local_data().set_feed_persistence_disabled(true) {
            tracing::warn!(error = %error, "failed to disable server feed logging");
        } else {
            tracing::info!("server feed logging disabled (desktop is the feed recorder)");
        }
    }

    let listen_addr: SocketAddr = match config.listen_addr.parse() {
        Ok(addr) => addr,
        Err(error) => {
            tracing::error!(error = %error, addr = %config.listen_addr, "invalid listen address");
            shutdown(&state, "invalid-listen-addr");
            return ExitCode::from(1);
        }
    };
    let registry_len = ctx.registry.len();
    let app = axum::Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .with_state(Arc::clone(&state))
        .route(
            "/api/public/profile",
            axum::routing::get(transport::public::profile),
        )
        .route(
            "/api/public/mutual-friends",
            axum::routing::get(transport::public::mutual_friends),
        )
        .route(
            "/api/invoke",
            axum::routing::post(transport::invoke::invoke_endpoint),
        )
        .route(
            "/api/admin/status",
            axum::routing::get(transport::admin_auth::status_endpoint),
        )
        .route(
            "/api/admin/auth",
            axum::routing::post(transport::admin_auth::auth_endpoint),
        )
        .route(
            "/api/admin/logout",
            axum::routing::post(transport::admin_auth::logout_endpoint),
        )
        .route(
            "/api/events",
            axum::routing::get(transport::events::events_endpoint),
        )
        .route(
            "/api/img/{file_id}/{version}",
            axum::routing::get(transport::img::img_endpoint),
        )
        .with_state(Arc::clone(&ctx))
        .fallback(transport::static_files::static_endpoint)
        .with_state(transport::static_files::StaticFiles::new(
            config.web.dist_dir.clone(),
        ));
    let listener = match tokio::net::TcpListener::bind(listen_addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(error = %error, addr = %listen_addr, "failed to bind listen address");
            shutdown(&state, "bind-failed");
            return ExitCode::from(1);
        }
    };
    tracing::info!(
        addr = %listen_addr,
        commands = registry_len,
        "HTTP server listening"
    );

    let serve = axum::serve(listener, app);
    tokio::select! {
        result = serve => {
            if let Err(error) = result {
                tracing::error!(error = %error, "HTTP server failed");
                shutdown(&state, "http-failed");
                return ExitCode::from(1);
            }
        }
        _ = shutdown_signal() => {
            tracing::info!("shutdown signal received");
            shutdown(&state, "signal");
        }
    }
    ExitCode::SUCCESS
}

async fn healthz(
    axum::extract::State(state): axum::extract::State<Arc<ServerRuntimeHostState>>,
) -> axum::response::Json<Value> {
    let snapshot = state.backend_runtime_snapshot();
    axum::response::Json(json!({
        "ok": true,
        "phase": format!("{:?}", snapshot.phase),
        "authStatus": format!("{:?}", snapshot.auth_status),
        "authUserId": snapshot.auth_user_id,
        "mode": format!("{:?}", snapshot.mode),
    }))
}

/// Build the server's data-dir resolution. The server deliberately uses
/// its own directory namespace so it can run beside a desktop VRCX-0
/// install on the same machine without sharing profile locks.
fn resolve_server_app_data_dir(data_dir: &Path) -> AppDataDirResolution {
    AppDataDirResolution {
        current_dir: data_dir.to_path_buf(),
        default_dir: data_dir.to_path_buf(),
        persisted_dir: None,
        cli_dir: None,
        source: AppDataDirSource::Default,
    }
}

/// Write the sync connection fields provided by the server config into
/// the runtime's configs table before the sync engine starts. Only
/// provided fields are written, so UI-configured values survive boots
/// with a partial or absent [sync] section.
fn seed_sync_settings(
    state: &ServerRuntimeHostState,
    settings: &config::SyncSettings,
) -> Result<(), String> {
    let db = state.runtime().database();
    let mut wrote_anything = false;
    fn set_optional(
        db: &Arc<vrcx_0_persistence::DatabaseService>,
        key: &str,
        value: Option<String>,
    ) -> Result<bool, String> {
        if let Some(value) = value {
            if !value.trim().is_empty() {
                set_string(db, key, &value).map_err(|error| error.to_string())?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    wrote_anything |= set_optional(db, CONFIG_HOST, settings.host.clone())?;
    if let Some(port) = settings.port {
        if (1..65536).contains(&port) {
            set_string(db, CONFIG_PORT, &port.to_string()).map_err(|e| e.to_string())?;
            wrote_anything = true;
        }
    }
    wrote_anything |= set_optional(db, CONFIG_USER, settings.user.clone())?;
    wrote_anything |= set_optional(db, CONFIG_PASSWORD, settings.password.clone())?;
    wrote_anything |= set_optional(db, CONFIG_DATABASE, settings.database.clone())?;
    if let Some(tls_verify) = settings.tls_verify {
        set_bool(db, CONFIG_TLS_VERIFY, tls_verify).map_err(|e| e.to_string())?;
        wrote_anything = true;
    }
    if let Some(allow_plaintext) = settings.allow_plaintext {
        set_bool(db, CONFIG_ALLOW_PLAINTEXT, allow_plaintext).map_err(|e| e.to_string())?;
        wrote_anything = true;
    }
    if let Some(interval) = settings.interval_sec {
        if (5..=3600).contains(&interval) {
            set_string(db, CONFIG_INTERVAL_SEC, &interval.to_string())
                .map_err(|e| e.to_string())?;
            wrote_anything = true;
        }
    }

    if settings.is_complete() {
        set_bool(db, CONFIG_ENABLED, true).map_err(|e| e.to_string())?;
        wrote_anything = true;
        tracing::info!(
            host = settings.host.as_deref().unwrap_or_default(),
            database = settings.database.as_deref().unwrap_or_default(),
            interval_sec = settings.interval_sec.unwrap_or(DEFAULT_INTERVAL_SEC as i64),
            "remote sync enabled from server config"
        );
    }
    if wrote_anything {
        tracing::debug!("seeded remote sync settings from server config");
    }
    Ok(())
}

fn spawn_activity_refresh(state: Arc<ServerRuntimeHostState>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
            let scope = state.auth_scope_snapshot();
            if !scope.active || scope.current_user_id.is_empty() {
                continue;
            }
            let owner = vrcx_0_core::OwnerId::new(scope.current_user_id.clone());
            let local_data = state.local_data().clone();
            let refresh = tokio::task::spawn_blocking(move || {
                let refresh_input =
                    vrcx_0_persistence::activity::ActivitySelfSessionsRefreshInput {
                        user_id: owner.as_str().to_string(),
                        mode: vrcx_0_persistence::activity::ActivityRefreshMode::Incremental,
                        range_days: serde_json::json!(365i64),
                        now_ms: None,
                    };
                vrcx_0_persistence::activity::activity_self_sessions_refresh(
                    local_data.database(),
                    &owner,
                    refresh_input,
                )?;
                for range_days in [30i64, 90, 180, 365] {
                    let _ = local_data.activity_view(
                        vrcx_0_persistence::activity::ActivityViewBuildInput {
                            owner_user_id: owner.clone(),
                            target_user_id: owner.as_str().to_string(),
                            is_self: true,
                            range_days,
                            utc_offset_minutes: 0,
                            now_ms: 0,
                            force_refresh: true,
                        },
                    );
                }
                Ok::<(), vrcx_0_persistence::Error>(())
            })
            .await;
            match refresh {
                Ok(Ok(())) => tracing::debug!("activity refresh cycle completed"),
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "activity refresh failed")
                }
                Err(error) => {
                    tracing::warn!(error = %error, "activity refresh task failed")
                }
            }
        }
    });
}

fn shutdown(state: &Arc<ServerRuntimeHostState>, reason: &str) {
    state.stop_for_application_exit(reason);
    state.release_profile_lock();
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}

struct TokioRuntimeTaskExecutor;

struct TokioRuntimeTaskHandle(tokio::task::JoinHandle<()>);

impl RuntimeTaskExecutor for TokioRuntimeTaskExecutor {
    fn spawn(&self, task: RuntimeTask) -> Box<dyn RuntimeTaskHandle> {
        Box::new(TokioRuntimeTaskHandle(tokio::spawn(task)))
    }
}

impl RuntimeTaskHandle for TokioRuntimeTaskHandle {
    fn abort(&self) {
        self.0.abort();
    }

    fn is_finished(&self) -> bool {
        self.0.is_finished()
    }

    fn join_or_abort(&mut self, timeout: Duration) {
        if self.is_finished() {
            let _ = block_on_runtime_task(&mut self.0);
            return;
        }

        let Some(joined) =
            block_on_runtime_task(async { tokio::time::timeout(timeout, &mut self.0).await })
        else {
            self.0.abort();
            return;
        };
        if joined.is_ok() {
            return;
        }

        self.0.abort();
        let _ = block_on_runtime_task(async {
            tokio::time::timeout(Duration::from_millis(50), &mut self.0).await
        });
    }
}

/// Block on a runtime task from async context; safe on the multi-thread
/// runtime via `block_in_place`, mirroring the headless executor.
fn block_on_runtime_task<F>(future: F) -> Option<F::Output>
where
    F: std::future::Future,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            Some(tokio::task::block_in_place(|| handle.block_on(future)))
        }
        Ok(_) => None,
        Err(_) => None,
    }
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,vrcx_0_server=debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn product_app_version() -> String {
    const PACKAGE_JSON: &str = include_str!("../../../package.json");
    serde_json::from_str::<Value>(PACKAGE_JSON)
        .ok()
        .and_then(|value| {
            value
                .get("version")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|version| !version.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").into())
}

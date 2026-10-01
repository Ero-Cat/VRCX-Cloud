//! Server configuration: a TOML file next to the data directory plus
//! `VRCX_CLOUD_*` environment overrides.
//!
//! Sync connection fields provided here are seeded into the runtime's
//! SQLite `configs` table on boot (before the remote sync engine starts),
//! so the server joins the user's sync mesh without touching the UI.
//! Fields left empty never overwrite values already configured there.

use std::path::PathBuf;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8800";

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerConfigFile {
    pub server: Option<ServerSection>,
    pub sync: Option<SyncSection>,
    pub web: Option<WebSection>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ServerSection {
    /// Data directory holding the SQLite profile and caches. Defaults to
    /// `<system config dir>/VRCX-0-Server`, kept separate from a desktop
    /// VRCX-0 install on the same machine.
    pub data_dir: Option<String>,
    /// HTTP listen address for the web server.
    pub listen_addr: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct WebSection {
    /// Password for the web UI (single account). Also settable via
    /// `VRCX_CLOUD_WEB_PASSWORD`. When neither is set the web layer is
    /// open — only acceptable on a trusted LAN.
    pub password: Option<String>,
    /// Explicitly disable web authentication (trusted LAN only).
    pub auth_disabled: Option<bool>,
    /// Directory with the built frontend (index.html + assets).
    /// Also settable via `VRCX_CLOUD_DIST_DIR`; defaults to `./dist`.
    pub dist_dir: Option<String>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct SyncSection {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub tls_verify: Option<bool>,
    /// Explicit opt-in for unencrypted connections (trusted LAN only).
    pub allow_plaintext: Option<bool>,
    /// Sync cadence in seconds (5..=3600, default 60).
    pub interval_sec: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub listen_addr: String,
    pub sync: SyncSettings,
    pub web: WebSettings,
}

/// Sync settings resolved from file+env; `None` fields are not seeded.
#[derive(Clone, Debug, Default)]
pub struct SyncSettings {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub database: Option<String>,
    pub tls_verify: Option<bool>,
    pub allow_plaintext: Option<bool>,
    pub interval_sec: Option<i64>,
}

impl SyncSettings {
    pub fn is_complete(&self) -> bool {
        self.host.as_deref().is_some_and(|v| !v.trim().is_empty())
            && self.user.as_deref().is_some_and(|v| !v.trim().is_empty())
            && self.password.as_deref().is_some_and(|v| !v.is_empty())
            && self
                .database
                .as_deref()
                .is_some_and(|v| !v.trim().is_empty())
    }
}

impl ServerConfig {
    /// Load `server.toml` (path from `VRCX_CLOUD_CONFIG`, default
    /// `<data_dir>/server.toml`) and apply `VRCX_CLOUD_*` env overrides.
    pub fn load() -> Result<Self, String> {
        let file: ServerConfigFile = std::env::var("VRCX_CLOUD_CONFIG")
            .ok()
            .and_then(|path| {
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|content| (path, content))
            })
            .map(|(path, content)| {
                toml::from_str(&content)
                    .map_err(|error| format!("invalid server config {path}: {error}"))
            })
            .transpose()?
            .unwrap_or_default();

        let server_section = file.server.unwrap_or_default();
        let data_dir = env_non_empty("VRCX_CLOUD_DATA_DIR")
            .or(server_section.data_dir)
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let listen_addr = env_non_empty("VRCX_CLOUD_LISTEN")
            .or(server_section.listen_addr)
            .unwrap_or_else(|| DEFAULT_LISTEN_ADDR.to_string());

        let sync_file = file.sync.unwrap_or_default();
        let sync = SyncSettings {
            host: env_non_empty("VRCX_CLOUD_SYNC_HOST").or(sync_file.host),
            port: env_i64("VRCX_CLOUD_SYNC_PORT").or(sync_file.port),
            user: env_non_empty("VRCX_CLOUD_SYNC_USER").or(sync_file.user),
            password: env_non_empty("VRCX_CLOUD_SYNC_PASSWORD").or(sync_file.password),
            database: env_non_empty("VRCX_CLOUD_SYNC_DATABASE").or(sync_file.database),
            tls_verify: env_bool("VRCX_CLOUD_SYNC_TLS_VERIFY").or(sync_file.tls_verify),
            allow_plaintext: env_bool("VRCX_CLOUD_SYNC_ALLOW_PLAINTEXT")
                .or(sync_file.allow_plaintext),
            interval_sec: env_i64("VRCX_CLOUD_SYNC_INTERVAL_SEC").or(sync_file.interval_sec),
        };

        let web_file = file.web.unwrap_or_default();
        let auth_disabled = env_bool("VRCX_CLOUD_WEB_AUTH_DISABLED")
            .or(web_file.auth_disabled)
            .unwrap_or(false);
        let password = env_non_empty("VRCX_CLOUD_WEB_PASSWORD").or(web_file.password);
        let web = WebSettings {
            // Auth is enabled when a password exists unless explicitly
            // disabled; with no password at all the UI is open (LAN trust).
            auth_enabled: !auth_disabled && password.is_some(),
            password,
            dist_dir: env_non_empty("VRCX_CLOUD_DIST_DIR")
                .or(web_file.dist_dir)
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("dist")),
        };

        Ok(Self {
            data_dir,
            listen_addr,
            sync,
            web,
        })
    }
}

/// Web UI access settings.
#[derive(Clone, Debug)]
pub struct WebSettings {
    pub auth_enabled: bool,
    pub password: Option<String>,
    pub dist_dir: PathBuf,
}

fn default_data_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VRCX-0-Server")
}

fn env_non_empty(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|v| {
        let trimmed = v.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn env_i64(key: &str) -> Option<i64> {
    env_non_empty(key).and_then(|v| v.parse().ok())
}

fn env_bool(key: &str) -> Option<bool> {
    env_non_empty(key).map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

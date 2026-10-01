//! Desktop-activity supervisor: polls remote-sync device presence and
//! drives the realtime pause gate. A desktop counts as active while it
//! pushed or pulled sync ops inside the activity window; a few seconds
//! after it goes quiet the server resumes its own realtime session.

use std::sync::Arc;
use std::time::Duration;

use vrcx_0_contracts::SyncDeviceRecord;
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use crate::realtime_gate::PauseGate;

const POLL_INTERVAL: Duration = Duration::from_secs(30);
/// How recently a device must have synced to count as active. Generous
/// versus the default 60s sync interval, and covers 15s tinkerers too.
const ACTIVITY_WINDOW: Duration = Duration::from_secs(90);

pub fn spawn(state: Arc<ServerRuntimeHostState>, gate: PauseGate) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            let status = state.remote_sync().status().await;
            let my_device = status.device_id;
            let desktop_active = status
                .remote_devices
                .iter()
                .any(|device| device_is_active_desktop(device, &my_device));

            if gate.set_paused(desktop_active) {
                if desktop_active {
                    tracing::info!(
                        devices = ?status.remote_devices.iter().map(|d| d.device_id.as_str()).collect::<Vec<_>>(),
                        "desktop device active - server realtime session will pause"
                    );
                } else {
                    tracing::info!("desktop devices quiet - server realtime session resumed");
                }
            }
        }
    });
}

fn device_is_active_desktop(device: &SyncDeviceRecord, own_device_id: &str) -> bool {
    if device.device_id == own_device_id {
        return false;
    }
    let last_seen = device
        .last_push_at
        .as_deref()
        .and_then(parse_iso)
        .into_iter()
        .chain(device.last_pull_at.as_deref().and_then(parse_iso))
        .max();
    match last_seen {
        Some(seen) => {
            let now = chrono::Utc::now().timestamp();
            (now - seen).abs() <= ACTIVITY_WINDOW.as_secs() as i64
        }
        None => false,
    }
}

fn parse_iso(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.timestamp())
}

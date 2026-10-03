//! Desktop-activity supervisor: polls remote-sync device presence and
//! drives the realtime pause gate. A desktop device counts as active
//! while it pushed or pulled sync ops inside the activity window; a few
//! seconds after it goes quiet the server resumes its own realtime
//! session. Server-profile devices never block each other the way
//! desktops do — instead they deterministically elect the fresh server
//! with the lowest device id as the single collector, so two servers
//! sharing one sync database cannot pause each other forever.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use vrcx_0_application_core::RuntimeEventPayload;
use vrcx_0_contracts::SyncDeviceRecord;
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use crate::realtime_gate::PauseGate;

/// Broadcast on the runtime event bus whenever the handoff flips, so the
/// web UI can show whether the server or the desktop device is currently
/// collecting from VRChat.
#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeHandoffState {
    pub desktop_active: bool,
    pub active_devices: Vec<String>,
}

impl RuntimeEventPayload for RealtimeHandoffState {
    const EVENT_NAME: &'static str = "realtimeHandoffState";
}

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
            let blocking_devices = devices_owning_collection(&my_device, &status.remote_devices);
            let desktop_active = !blocking_devices.is_empty();

            if gate.set_paused(desktop_active) {
                if desktop_active {
                    tracing::info!(
                        devices = ?blocking_devices,
                        "another device owns collection - server realtime session will pause"
                    );
                } else {
                    tracing::info!("other devices quiet - server realtime session resumed");
                }
            }
            // Broadcast every cycle so browsers connecting between
            // transitions still learn the current handoff state.
            state
                .runtime()
                .desktop_assembly()
                .event_bus()
                .emit(RealtimeHandoffState {
                    desktop_active,
                    active_devices: blocking_devices,
                });
        }
    });
}

/// Devices that currently own VRChat data collection, so the server must
/// hold its own realtime session off. Non-self devices only:
///
/// - Desktop devices (profile `desktop`, or rows from installs predating
///   the profile column): always own collection while fresh — the
///   desktop is the primary collector by design.
/// - Server devices: never treat each other the way desktops are
///   treated — two servers sharing one sync database would otherwise
///   pause each other forever and nothing would collect. Instead all
///   servers compute the same deterministic winner (the fresh server
///   with the lowest device id); only that one collects.
fn devices_owning_collection(own_device_id: &str, devices: &[SyncDeviceRecord]) -> Vec<String> {
    let fresh: Vec<&SyncDeviceRecord> = devices
        .iter()
        .filter(|device| device_is_fresh(device, own_device_id))
        .collect();
    let desktops: Vec<String> = fresh
        .iter()
        .filter(|device| device.profile != vrcx_0_contracts::DEVICE_PROFILE_SERVER)
        .map(|device| device.device_id.clone())
        .collect();
    if !desktops.is_empty() {
        return desktops;
    }
    fresh
        .iter()
        .filter(|device| device.device_id.as_str() < own_device_id)
        .map(|device| device.device_id.clone())
        .collect()
}

fn device_is_fresh(device: &SyncDeviceRecord, own_device_id: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::{devices_owning_collection, ACTIVITY_WINDOW};
    use vrcx_0_contracts::{SyncDeviceRecord, DEVICE_PROFILE_SERVER};

    fn device(device_id: &str, last_seen_ago_secs: Option<i64>, profile: &str) -> SyncDeviceRecord {
        let stamp = last_seen_ago_secs.map(|ago| {
            chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - ago, 0)
                .unwrap_or_else(chrono::Utc::now)
                .to_rfc3339()
        });
        SyncDeviceRecord {
            device_id: device_id.to_string(),
            app_version: "test".to_string(),
            last_push_at: stamp.clone(),
            last_pull_at: stamp,
            profile: profile.to_string(),
        }
    }

    #[test]
    fn fresh_desktop_device_owns_collection() {
        let blockers =
            devices_owning_collection("server", &[device("desktop", Some(15), "desktop")]);
        assert_eq!(blockers, vec!["desktop".to_string()]);
    }

    #[test]
    fn quiet_desktop_device_does_not_block() {
        let window = ACTIVITY_WINDOW.as_secs() as i64;
        assert!(devices_owning_collection(
            "server",
            &[device("desktop", Some(window + 1), "desktop")]
        )
        .is_empty());
    }

    #[test]
    fn own_device_never_blocks() {
        assert!(
            devices_owning_collection("server", &[device("server", Some(0), "server")]).is_empty()
        );
    }

    #[test]
    fn peer_server_with_lower_device_id_wins_collection() {
        let blockers =
            devices_owning_collection("b00d", &[device("aaaa", Some(10), DEVICE_PROFILE_SERVER)]);
        assert_eq!(blockers, vec!["aaaa".to_string()]);
    }

    #[test]
    fn peer_server_with_higher_device_id_does_not_block() {
        assert!(devices_owning_collection(
            "aaaa",
            &[device("b00d", Some(10), DEVICE_PROFILE_SERVER)]
        )
        .is_empty());
    }

    /// Two servers sharing one sync database must not pause each other
    /// forever: the deterministic winner collects, the other defers.
    #[test]
    fn two_servers_elect_exactly_one_collector() {
        let devices = [
            device("aaaa", Some(5), DEVICE_PROFILE_SERVER),
            device("b00d", Some(5), DEVICE_PROFILE_SERVER),
        ];
        let a_blocks = devices_owning_collection("aaaa", &devices);
        let b_blocks = devices_owning_collection("b00d", &devices);
        assert!(a_blocks.is_empty(), "lowest id collects");
        assert_eq!(b_blocks, vec!["aaaa".to_string()], "higher id defers");
    }

    #[test]
    fn desktop_outranks_server_election() {
        let devices = [
            device("aaaa", Some(5), DEVICE_PROFILE_SERVER),
            device("ffff", Some(5), "desktop"),
        ];
        // Both servers defer to the desktop, regardless of the election.
        assert_eq!(
            devices_owning_collection("0000", &devices),
            vec!["ffff".to_string()]
        );
    }

    #[test]
    fn legacy_row_without_profile_acts_as_desktop() {
        let mut legacy = device("legacy", Some(10), "desktop");
        legacy.profile = String::new();
        assert_eq!(
            devices_owning_collection("server", &[legacy]),
            vec!["legacy".to_string()]
        );
    }

    #[test]
    fn device_without_timestamps_is_inactive() {
        assert!(
            devices_owning_collection("server", &[device("desktop", None, "desktop")]).is_empty()
        );
    }
}

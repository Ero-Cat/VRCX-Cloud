use std::sync::Mutex;

use serde::Serialize;
use specta::Type;
use vrcx_0_core::OwnerId;
use vrcx_0_runtime_host_desktop::notification::{
    DesktopNotificationAction, DesktopNotificationTarget,
};

#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use tauri::{Emitter, Manager};

#[cfg(windows)]
use crate::state::AppState;

#[cfg(windows)]
pub const DESKTOP_NOTIFICATION_ACTIVATED_EVENT: &str = "desktopNotificationActivated";
#[cfg(windows)]
const DESKTOP_NOTIFICATION_ACTIVATION_DELAY: Duration = Duration::from_millis(300);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DesktopNotificationActivation {
    pub target: DesktopNotificationTarget,
}

#[derive(Default)]
pub struct PendingDesktopNotificationActivations {
    state: Mutex<PendingDesktopNotificationActivationState>,
}

#[derive(Default)]
struct PendingDesktopNotificationActivationState {
    #[cfg(any(windows, test))]
    generation: u64,
    #[cfg(any(windows, test))]
    scheduled: Option<DesktopNotificationAction>,
    ready: Option<DesktopNotificationAction>,
}

impl PendingDesktopNotificationActivations {
    #[cfg(any(windows, test))]
    pub fn replace(&self, action: DesktopNotificationAction) -> u64 {
        let Ok(mut state) = self.state.lock() else {
            return 0;
        };
        state.generation = state.generation.wrapping_add(1).max(1);
        state.scheduled = Some(action);
        state.ready = None;
        state.generation
    }

    #[cfg(any(windows, test))]
    pub fn promote_if_latest(&self, generation: u64) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if generation == 0 || state.generation != generation {
            return false;
        }
        state.ready = state.scheduled.take();
        state.ready.is_some()
    }

    pub fn take_for_owner(&self, owner_user_id: &OwnerId) -> Option<DesktopNotificationActivation> {
        let action = self.state.lock().ok()?.ready.take()?;
        if &action.owner_user_id != owner_user_id {
            return None;
        }
        Some(DesktopNotificationActivation {
            target: action.target,
        })
    }
}

#[cfg(windows)]
pub(crate) fn queue_desktop_notification_activation(
    app: &tauri::AppHandle,
    action: DesktopNotificationAction,
) {
    let Some(state) = app.try_state::<AppState>() else {
        tracing::warn!("ignored desktop notification activation before app state was ready");
        return;
    };
    let generation = state
        .pending_desktop_notification_activations()
        .replace(action);
    if generation == 0 {
        tracing::warn!("failed to queue desktop notification activation");
        return;
    }

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(DESKTOP_NOTIFICATION_ACTIVATION_DELAY).await;
        let Some(state) = app_handle.try_state::<AppState>() else {
            return;
        };
        if !state
            .pending_desktop_notification_activations()
            .promote_if_latest(generation)
        {
            return;
        }

        let main_thread_handle = app_handle.clone();
        if let Err(error) = app_handle.run_on_main_thread(move || {
            show_main_window_for_desktop_notification(&main_thread_handle);
            if let Err(error) =
                main_thread_handle.emit(DESKTOP_NOTIFICATION_ACTIVATED_EVENT, serde_json::json!({}))
            {
                tracing::warn!(error = %error, "failed to emit desktop notification activation");
            }
        }) {
            tracing::warn!(error = %error, "failed to schedule desktop notification window restore");
        }
    });
}

#[cfg(windows)]
fn show_main_window_for_desktop_notification(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if let Err(error) =
        crate::bootstrap::restore_foreground_window_from_background_mode(app, &state)
    {
        tracing::warn!(error = %error, "failed to show main window from desktop notification");
    }
}

#[cfg(test)]
mod tests {
    use vrcx_0_core::OwnerId;
    use vrcx_0_runtime_host_desktop::notification::{
        DesktopNotificationAction, DesktopNotificationTarget,
    };

    use super::PendingDesktopNotificationActivations;

    const OWNER_USER_ID: &str = "usr_12345678-1234-1234-1234-1234567890ab";
    const FIRST_USER_ID: &str = "usr_aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    const LAST_USER_ID: &str = "usr_bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

    #[test]
    fn rapid_replacements_promote_only_the_latest_target() {
        let pending = PendingDesktopNotificationActivations::default();
        let first_generation = pending.replace(action(FIRST_USER_ID));
        let last_generation = pending.replace(action(LAST_USER_ID));

        assert!(!pending.promote_if_latest(first_generation));
        assert!(pending.promote_if_latest(last_generation));
        assert_eq!(
            pending
                .take_for_owner(&OwnerId::new(OWNER_USER_ID))
                .unwrap()
                .target,
            DesktopNotificationTarget::OpenUserProfile {
                user_id: LAST_USER_ID.into(),
            }
        );
        assert!(pending
            .take_for_owner(&OwnerId::new(OWNER_USER_ID))
            .is_none());
    }

    #[test]
    fn newer_click_invalidates_an_undrained_ready_target() {
        let pending = PendingDesktopNotificationActivations::default();
        let first_generation = pending.replace(action(FIRST_USER_ID));
        assert!(pending.promote_if_latest(first_generation));

        pending.replace(action(LAST_USER_ID));

        assert!(pending
            .take_for_owner(&OwnerId::new(OWNER_USER_ID))
            .is_none());
    }

    #[test]
    fn activation_is_discarded_after_the_authenticated_owner_changes() {
        let pending = PendingDesktopNotificationActivations::default();
        let generation = pending.replace(action(FIRST_USER_ID));
        assert!(pending.promote_if_latest(generation));

        assert!(pending
            .take_for_owner(&OwnerId::new("usr_cccccccc-cccc-cccc-cccc-cccccccccccc"))
            .is_none());
        assert!(pending
            .take_for_owner(&OwnerId::new(OWNER_USER_ID))
            .is_none());
    }

    fn action(user_id: &str) -> DesktopNotificationAction {
        DesktopNotificationAction::open_user_profile(&OwnerId::new(OWNER_USER_ID), user_id).unwrap()
    }
}

use std::sync::Arc;

use chrono::Utc;
use vrcx_0_application::social::{
    scan_next_profile_watch, ProfileWatchDeps, ProfileWatchOutcome, ProfileWatchPacer,
    ProfileWatchScanOutcome, BACKGROUND_PROFILE_WATCH_JOB, PROFILE_WATCH_CONFIG_KEY,
    PROFILE_WATCH_INTERVAL, PROFILE_WATCH_PAUSE,
};
use vrcx_0_core::OwnerId;

use super::super::{background_capability_session_identity, emit_background_warning};
use super::BackgroundTickContext;

pub(in crate::state) async fn run_background_profile_watch(
    context: &BackgroundTickContext<'_>,
    pacer: &ProfileWatchPacer,
) {
    let enabled = context
        .runtime_context
        .config()
        .get_bool(PROFILE_WATCH_CONFIG_KEY, false)
        .unwrap_or(false);
    let Some(session) =
        background_capability_session_identity(context.session_slot).filter(|_| enabled)
    else {
        context.background_jobs.mark_scheduled(
            BACKGROUND_PROFILE_WATCH_JOB,
            "Watched-profile polling is disabled or waiting for an authenticated session.",
            PROFILE_WATCH_INTERVAL.as_secs(),
        );
        return;
    };
    let now = Utc::now();
    context.background_jobs.mark_running(
        BACKGROUND_PROFILE_WATCH_JOB,
        "Polling the next watched profile.",
    );
    let store = vrcx_0_outbound_adapters::LocalProfileWatchStore::new(Arc::clone(context.db));
    let remote = vrcx_0_outbound_adapters::VrchatRequestAdapter::new(Arc::clone(context.web));
    let deps = ProfileWatchDeps {
        store: &store,
        remote_requests: &vrcx_0_outbound_adapters::VrchatProfileBioRemoteRequests,
        remote: &remote,
        realtime: context.realtime_runtime,
        pacer,
        owner: OwnerId::new(session.current_user_id),
        endpoint: session.endpoint,
    };
    match scan_next_profile_watch(&deps, now).await {
        Ok(outcome) => {
            context
                .background_jobs
                .mark_completed(BACKGROUND_PROFILE_WATCH_JOB, scan_detail(&outcome));
        }
        Err(error) => {
            tracing::warn!(error = %error, "background watched-profile poll failed");
            emit_background_warning(
                context.runtime_context,
                context.backend_runtime,
                format!("watched-profile poll failed: {error}."),
            );
            context
                .background_jobs
                .mark_failed(BACKGROUND_PROFILE_WATCH_JOB, error.to_string());
            pacer.pause(now);
        }
    }
    let delay = if pacer.is_paused(now) {
        PROFILE_WATCH_PAUSE
    } else {
        PROFILE_WATCH_INTERVAL
    };
    context.background_jobs.mark_scheduled(
        BACKGROUND_PROFILE_WATCH_JOB,
        "Next watched-profile poll is waiting.",
        delay.as_secs(),
    );
}

fn scan_detail(outcome: &ProfileWatchScanOutcome) -> String {
    match outcome {
        ProfileWatchScanOutcome::Paused => "watched-profile poll is paused.".into(),
        ProfileWatchScanOutcome::Idle => "no watched profile is due for a poll.".into(),
        ProfileWatchScanOutcome::Checked { user_id, outcome } => {
            let result = match outcome {
                ProfileWatchOutcome::Baseline => "first observation recorded".into(),
                ProfileWatchOutcome::Unchanged => "no change".into(),
                ProfileWatchOutcome::Changed { published } => {
                    format!(
                        "{published} feed entr{} published",
                        if *published == 1 { "y" } else { "ies" }
                    )
                }
                ProfileWatchOutcome::Deferred => "change deferred".into(),
            };
            format!("polled {user_id}: {result}.")
        }
        ProfileWatchScanOutcome::Throttled { status } => {
            format!("VRChat pushed back with HTTP {status}; poll paused.")
        }
        ProfileWatchScanOutcome::Unavailable { user_id, status } => {
            format!("profile {user_id} unavailable (HTTP {status}).")
        }
    }
}

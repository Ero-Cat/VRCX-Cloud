//! Non-friend profile watching: polls a user-curated list of watched
//! (non-friend) user ids on the public profile endpoint, diffs bio /
//! status / status description against the last observation, and publishes
//! changes as ordinary feed events so the timeline, history tabs, and
//! cross-device sync all work unchanged. Disabled unless the
//! `profileWatchEnabled` config key is set.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use vrcx_0_application_core::vrchat_api::{
    classify_api_response, ApiResponseClass, VrchatApiResponse, VrchatScope,
};
use vrcx_0_application_core::Result;
use vrcx_0_application_realtime::RealtimeHostRuntime;
use vrcx_0_contracts::feed_live::FeedLiveEntry;
use vrcx_0_contracts::vrchat_api::VrchatJsonResponse;
use vrcx_0_core::json::JsonExt;
use vrcx_0_core::time::iso_millis;
use vrcx_0_core::OwnerId;

use crate::remote::VrchatRequestPort;

#[cfg(test)]
mod tests;
use crate::social::profile_bio::ProfileBioRemoteRequests;

pub const PROFILE_WATCH_CONFIG_KEY: &str = "profileWatchEnabled";
/// One poll per maintenance tick while work is available.
pub const PROFILE_WATCH_INTERVAL: Duration = Duration::from_secs(60);
/// Each watched user is re-checked at most this often.
pub const PROFILE_WATCH_MIN_AGE: Duration = Duration::from_secs(5 * 60);
pub const PROFILE_WATCH_PAUSE: Duration = Duration::from_secs(10 * 60);

pub trait ProfileWatchStore: Send + Sync {
    fn next_stale(&self, owner: &OwnerId, checked_before: &str) -> Result<Option<String>>;
    fn snapshot(&self, owner: &OwnerId, user_id: &str) -> Result<Option<ProfileWatchSeen>>;
    fn record(
        &self,
        owner: &OwnerId,
        user_id: &str,
        observation: &ProfileWatchObservation,
        polled_at: &str,
    ) -> Result<()>;
    fn mark_checked(&self, owner: &OwnerId, user_id: &str, checked_at: &str) -> Result<()>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProfileWatchSeen {
    pub display_name: String,
    pub status: String,
    pub status_description: String,
    pub bio: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileWatchObservation {
    pub user_id: String,
    pub display_name: String,
    pub status: String,
    pub status_description: String,
    pub bio: String,
}

impl ProfileWatchObservation {
    pub fn from_profile(profile: &Value) -> Option<Self> {
        let user_id = profile.trimmed_text("id");
        if !user_id.starts_with("usr_") {
            return None;
        }
        Some(Self {
            user_id,
            display_name: profile.trimmed_text("displayName"),
            status: profile.trimmed_text("status"),
            status_description: profile.trimmed_text("statusDescription"),
            bio: profile
                .get("bio")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string(),
        })
    }

    fn from_response(response: &VrchatApiResponse) -> Option<Self> {
        if classify_api_response(response.status).class != ApiResponseClass::Ok {
            return None;
        }
        Self::from_profile(&VrchatJsonResponse::from(response).json)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileWatchOutcome {
    /// First observation recorded without publishing (baseline).
    Baseline,
    /// Nothing changed; the poll timestamp moved.
    Unchanged,
    /// Feed entries were published for the changed fields.
    Changed { published: usize },
    /// The realtime session was not current; retry next tick.
    Deferred,
}

/// Diff one observation against the stored snapshot and publish feed
/// entries through the non-friend publish path.
pub fn observe_profile_watch(
    store: &dyn ProfileWatchStore,
    realtime: &Arc<RealtimeHostRuntime>,
    owner: &OwnerId,
    observation: &ProfileWatchObservation,
    now: &str,
) -> Result<ProfileWatchOutcome> {
    let display_name = if observation.display_name.is_empty() {
        observation.user_id.clone()
    } else {
        observation.display_name.clone()
    };
    let Some(seen) = store.snapshot(owner, &observation.user_id)? else {
        store.record(owner, &observation.user_id, observation, now)?;
        return Ok(ProfileWatchOutcome::Baseline);
    };
    if seen.bio == observation.bio
        && seen.status == observation.status
        && seen.status_description == observation.status_description
    {
        store.record(owner, &observation.user_id, observation, now)?;
        return Ok(ProfileWatchOutcome::Unchanged);
    }

    let mut entries = Vec::new();
    if seen.bio != observation.bio {
        entries.push(FeedLiveEntry::Bio {
            created_at: now.to_string(),
            user_id: observation.user_id.clone(),
            display_name: display_name.clone(),
            bio: observation.bio.clone(),
            previous_bio: seen.bio.clone(),
            owner_user_id: String::new(),
        });
    }
    if seen.status != observation.status
        || seen.status_description != observation.status_description
    {
        entries.push(FeedLiveEntry::Status {
            created_at: now.to_string(),
            user_id: observation.user_id.clone(),
            display_name,
            status: observation.status.clone(),
            status_description: observation.status_description.clone(),
            previous_status: seen.status.clone(),
            previous_status_description: seen.status_description.clone(),
            owner_user_id: String::new(),
        });
    }
    let mut published = 0;
    for entry in entries {
        if !realtime.publish_external_feed_entry(owner, entry) {
            return Ok(ProfileWatchOutcome::Deferred);
        }
        published += 1;
    }
    store.record(owner, &observation.user_id, observation, now)?;
    Ok(ProfileWatchOutcome::Changed { published })
}

#[derive(Default)]
pub struct ProfileWatchPacer {
    paused_until: Mutex<Option<DateTime<Utc>>>,
}

impl ProfileWatchPacer {
    pub fn is_paused(&self, now: DateTime<Utc>) -> bool {
        self.paused_until
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_some_and(|until| now < until)
    }

    pub fn pause(&self, now: DateTime<Utc>) {
        *self
            .paused_until
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(now + PROFILE_WATCH_PAUSE);
    }
}

pub struct ProfileWatchDeps<'a> {
    pub store: &'a dyn ProfileWatchStore,
    pub remote_requests: &'a dyn ProfileBioRemoteRequests,
    pub remote: &'a dyn VrchatRequestPort,
    pub realtime: &'a Arc<RealtimeHostRuntime>,
    pub pacer: &'a ProfileWatchPacer,
    pub owner: OwnerId,
    pub endpoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileWatchScanOutcome {
    Paused,
    Idle,
    Checked {
        user_id: String,
        outcome: ProfileWatchOutcome,
    },
    Throttled {
        status: i32,
    },
    Unavailable {
        user_id: String,
        status: i32,
    },
}

pub async fn scan_next_profile_watch(
    deps: &ProfileWatchDeps<'_>,
    now: DateTime<Utc>,
) -> Result<ProfileWatchScanOutcome> {
    if deps.pacer.is_paused(now) {
        return Ok(ProfileWatchScanOutcome::Paused);
    }
    let checked_before = iso_millis(now - PROFILE_WATCH_MIN_AGE);
    let Some(user_id) = deps.store.next_stale(&deps.owner, &checked_before)? else {
        deps.pacer.pause(now);
        return Ok(ProfileWatchScanOutcome::Idle);
    };
    let request = deps
        .remote_requests
        .profile(deps.endpoint.clone(), user_id.clone())?;
    let response = deps.remote.send(request, VrchatScope::Vrchat).await?;
    let now_iso = iso_millis(now);
    let observation = match classify_api_response(response.status).class {
        ApiResponseClass::Ok => ProfileWatchObservation::from_response(&response),
        ApiResponseClass::ClientError => None,
        ApiResponseClass::Auth
        | ApiResponseClass::RateLimited
        | ApiResponseClass::ServerError
        | ApiResponseClass::Unknown => {
            deps.pacer.pause(now);
            return Ok(ProfileWatchScanOutcome::Throttled {
                status: response.status,
            });
        }
    };
    let Some(observation) = observation else {
        deps.store.mark_checked(&deps.owner, &user_id, &now_iso)?;
        return Ok(ProfileWatchScanOutcome::Unavailable {
            user_id,
            status: response.status,
        });
    };
    let outcome = observe_profile_watch(
        deps.store,
        deps.realtime,
        &deps.owner,
        &observation,
        &now_iso,
    )?;
    Ok(ProfileWatchScanOutcome::Checked { user_id, outcome })
}

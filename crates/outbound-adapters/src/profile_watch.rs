use std::sync::Arc;

use vrcx_0_application::social::{ProfileWatchObservation, ProfileWatchSeen, ProfileWatchStore};
use vrcx_0_application_core::Result;
use vrcx_0_core::OwnerId;
use vrcx_0_persistence::watched_users::{self, WatchedUserObservation};
use vrcx_0_persistence::DatabaseService;

#[derive(Clone)]
pub struct LocalProfileWatchStore {
    db: Arc<DatabaseService>,
}

impl LocalProfileWatchStore {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        Self { db }
    }
}

impl ProfileWatchStore for LocalProfileWatchStore {
    fn next_stale(&self, owner: &OwnerId, checked_before: &str) -> Result<Option<String>> {
        watched_users::watched_user_next_stale(&self.db, owner.as_str(), checked_before)
            .map_err(crate::map_persistence_error)
    }

    fn snapshot(&self, owner: &OwnerId, user_id: &str) -> Result<Option<ProfileWatchSeen>> {
        watched_users::watched_user_snapshot(&self.db, owner.as_str(), user_id)
            .map(|snapshot| {
                snapshot.map(|seen| ProfileWatchSeen {
                    display_name: seen.display_name,
                    status: seen.status,
                    status_description: seen.status_description,
                    bio: seen.bio,
                })
            })
            .map_err(crate::map_persistence_error)
    }

    fn record(
        &self,
        owner: &OwnerId,
        user_id: &str,
        observation: &ProfileWatchObservation,
        polled_at: &str,
    ) -> Result<()> {
        watched_users::watched_user_record_observation(
            &self.db,
            owner.as_str(),
            user_id,
            &WatchedUserObservation {
                display_name: &observation.display_name,
                status: &observation.status,
                status_description: &observation.status_description,
                bio: &observation.bio,
                polled_at,
            },
        )
        .map_err(crate::map_persistence_error)
    }

    fn mark_checked(&self, owner: &OwnerId, user_id: &str, checked_at: &str) -> Result<()> {
        watched_users::watched_user_mark_polled(&self.db, owner.as_str(), user_id, checked_at)
            .map_err(crate::map_persistence_error)
    }
}

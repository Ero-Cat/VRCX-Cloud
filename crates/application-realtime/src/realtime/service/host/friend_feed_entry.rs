use std::sync::Arc;

use vrcx_0_contracts::feed_live::FeedLiveEntry;
use vrcx_0_core::OwnerId;

use super::fanout::FriendOutputApplyOutcome;
use super::state::{ActiveRealtimeContext, RealtimeHostRuntime};
use crate::realtime::RealtimeFriendOutput;

impl RealtimeHostRuntime {
    pub fn publish_friend_feed_entry(
        self: &Arc<Self>,
        owner_user_id: &OwnerId,
        entry: FeedLiveEntry,
    ) -> bool {
        let Some(active) = self
            .active_current_user_context()
            .filter(|active| active.session.user_id == owner_user_id.as_str())
        else {
            return false;
        };
        self.publish_friend_feed_entry_for(&active, entry)
    }

    pub(super) fn publish_friend_feed_entry_for(
        self: &Arc<Self>,
        active: &ActiveRealtimeContext,
        entry: FeedLiveEntry,
    ) -> bool {
        let Some(output) = self.friends.feed_entry_output(active.generation, entry) else {
            return false;
        };
        let owner = self.lock_friend_owner();
        matches!(
            self.apply_friend_output_owned(&owner, output),
            FriendOutputApplyOutcome::Applied { .. }
        )
    }
}

impl RealtimeHostRuntime {
    /// Feed entry for a user outside the friend roster (watched non-friend
    /// profiles). Follows the same persist + live-emit path as friend
    /// entries but skips the roster binding: the output only carries feed
    /// persistence, and generation currency is all apply checks.
    pub fn publish_external_feed_entry(
        self: &Arc<Self>,
        owner_user_id: &OwnerId,
        entry: FeedLiveEntry,
    ) -> bool {
        let Some(active) = self
            .active_current_user_context()
            .filter(|active| active.session.user_id == owner_user_id.as_str())
        else {
            return false;
        };
        let mut output = RealtimeFriendOutput::new(
            OwnerId::new(owner_user_id.as_str().to_string()),
            active.generation,
            0,
        );
        output.persistence.feed_entries.push(entry);
        let owner = self.lock_friend_owner();
        matches!(
            self.apply_friend_output_owned(&owner, output),
            FriendOutputApplyOutcome::Applied { .. }
        )
    }
}

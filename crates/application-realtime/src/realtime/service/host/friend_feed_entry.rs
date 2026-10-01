use std::sync::Arc;

use vrcx_0_contracts::feed_live::FeedLiveEntry;
use vrcx_0_core::OwnerId;

use super::fanout::FriendOutputApplyOutcome;
use super::state::{ActiveRealtimeContext, RealtimeHostRuntime};

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

use std::sync::Arc;

use vrcx_0_contracts::feed_live::FeedLiveEntry;
use vrcx_0_contracts::FileMetadataOutput;
use vrcx_0_core::files::extract_file_id;

use crate::realtime::FriendIconChange;

use super::state::{ActiveRealtimeContext, RealtimeHostRuntime};

impl RealtimeHostRuntime {
    pub(super) fn schedule_friend_icon_changes(
        self: &Arc<Self>,
        generation: u64,
        changes: Vec<FriendIconChange>,
    ) {
        if changes.is_empty() {
            return;
        }
        let Some(active) = self
            .active_current_user_context()
            .filter(|active| active.generation == generation)
        else {
            return;
        };
        for change in changes {
            let runtime = Arc::clone(self);
            let active = active.clone();
            self.deps.tasks.spawn(async move {
                runtime.resolve_friend_icon_change(active, change).await;
            });
        }
    }

    async fn resolve_friend_icon_change(
        self: Arc<Self>,
        active: ActiveRealtimeContext,
        change: FriendIconChange,
    ) {
        let endpoint = &active.session.endpoint;
        let Some(next) = self
            .resolve_avatar_image_file(endpoint, &change.next_icon_url)
            .await
        else {
            return;
        };
        let previous = self
            .resolve_avatar_image_file(endpoint, &change.previous_icon_url)
            .await;
        self.publish_friend_feed_entry_for(
            &active,
            avatar_feed_entry(&change, previous.as_ref(), &next),
        );
    }

    async fn resolve_avatar_image_file(
        &self,
        endpoint: &str,
        icon_url: &str,
    ) -> Option<FileMetadataOutput> {
        let file_id = extract_file_id(icon_url)?;
        self.deps
            .file_cache
            .resolve(&self.deps.web, endpoint, &file_id)
            .await
            .filter(|file| file.avatar_name.is_some())
    }
}

fn avatar_feed_entry(
    change: &FriendIconChange,
    previous: Option<&FileMetadataOutput>,
    next: &FileMetadataOutput,
) -> FeedLiveEntry {
    FeedLiveEntry::Avatar {
        created_at: change.created_at.clone(),
        user_id: change.user_id.clone(),
        display_name: change.display_name.clone(),
        owner_id: next.owner_id.clone(),
        previous_owner_id: previous
            .map(|file| file.owner_id.clone())
            .unwrap_or_default(),
        avatar_name: next.avatar_name.clone().unwrap_or_default(),
        previous_avatar_name: previous
            .and_then(|file| file.avatar_name.clone())
            .unwrap_or_default(),
        current_avatar_image_url: change.next_icon_url.clone(),
        previous_current_avatar_image_url: previous
            .map(|_| change.previous_icon_url.clone())
            .unwrap_or_default(),
        owner_user_id: String::new(),
    }
}

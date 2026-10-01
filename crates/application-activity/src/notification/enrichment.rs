use std::sync::{Arc, Mutex, Weak};

use crate::OverlayActivityDelivery;
use vrcx_0_core::location::{format_display_location, is_meaningful_world_name, parse_location};

use super::{CachedNotificationUserImageResolver, NotificationRemote};

#[derive(Clone, Default)]
pub struct RealtimeUserImageResolverSlot {
    inner: Arc<Mutex<Option<Weak<dyn CachedNotificationUserImageResolver>>>>,
}

impl RealtimeUserImageResolverSlot {
    pub fn set(&self, resolver: &Arc<dyn CachedNotificationUserImageResolver>) {
        match self.inner.lock() {
            Ok(mut slot) => {
                *slot = Some(Arc::downgrade(resolver));
            }
            Err(error) => {
                tracing::warn!("failed to set realtime user image resolver bridge: {error}");
            }
        }
    }

    pub fn cached_url(&self, endpoint: &str, user_id: &str) -> Option<String> {
        let resolver = self.inner.lock().ok()?.as_ref()?.upgrade()?;
        resolver.cached_url(endpoint, user_id)
    }
}

pub async fn resolve_delivery_world_name(
    remote: &dyn NotificationRemote,
    endpoint: &str,
    delivery: &OverlayActivityDelivery,
) -> Option<(String, String)> {
    if is_meaningful_world_name(&delivery.entry.content.world_name) {
        return None;
    }
    let world_id = {
        let content = &delivery.entry.content;
        let explicit = content.world_id.trim();
        if explicit.is_empty() {
            parse_location(&content.location).world_id
        } else {
            explicit.to_string()
        }
    };
    if world_id.is_empty() {
        return None;
    }
    let name = remote.world_name(endpoint, &world_id).await?;
    let parsed = parse_location(&delivery.entry.content.location);
    let display_location =
        format_display_location(&parsed, &name, &delivery.entry.content.group_name);
    Some((name, display_location))
}

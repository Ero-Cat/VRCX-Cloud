use std::time::Duration;

use crate::OverlayActivityDelivery;
use vrcx_0_core::location::parse_location;

use crate::notification::user_image::UserImageCache;
use crate::notification::NotificationRemote;
use vrcx_0_core::files::extract_file_id;

const DISCORD_RESOLVE_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) struct DiscordDeps<'a> {
    pub(crate) user_image_cache: &'a UserImageCache,
    pub(crate) remote: &'a dyn NotificationRemote,
    pub(crate) endpoint: &'a str,
}

pub(super) async fn resolve_avatar_name(
    deps: &DiscordDeps<'_>,
    delivery: &OverlayActivityDelivery,
) -> String {
    let Some(file_id) = extract_file_id(&delivery.entry.content.image_url) else {
        return String::new();
    };
    match tokio::time::timeout(
        DISCORD_RESOLVE_TIMEOUT,
        deps.remote.avatar_name(deps.endpoint, &file_id),
    )
    .await
    {
        Ok(Some(name)) => name,
        _ => String::new(),
    }
}

pub(super) async fn resolve_actor_icon_url(
    deps: &DiscordDeps<'_>,
    delivery: &OverlayActivityDelivery,
) -> String {
    let actor = delivery.entry.actor_user_id.trim();
    if actor.is_empty() {
        return String::new();
    }
    match tokio::time::timeout(
        DISCORD_RESOLVE_TIMEOUT,
        deps.user_image_cache
            .resolve(deps.remote, deps.endpoint, actor),
    )
    .await
    {
        Ok(result) => result.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

pub(super) async fn resolve_world_thumbnail_url(
    deps: &DiscordDeps<'_>,
    delivery: &OverlayActivityDelivery,
) -> String {
    let content = &delivery.entry.content;
    let explicit = content.world_id.trim();
    let world_id = if explicit.is_empty() {
        parse_location(&content.location).world_id
    } else {
        explicit.to_string()
    };
    if world_id.is_empty() {
        return String::new();
    }
    match tokio::time::timeout(
        DISCORD_RESOLVE_TIMEOUT,
        deps.remote.world_image_url(deps.endpoint, &world_id),
    )
    .await
    {
        Ok(Some(image_url)) => image_url,
        _ => String::new(),
    }
}

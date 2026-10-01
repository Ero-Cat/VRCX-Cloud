//! Cross-cutting services shared by the server runtime.
//!
//! Compared to the desktop services this keeps only what works without
//! access to the player's machine: the privacy lock, webhook-backed
//! overlay-activity fan-out, realtime notification image resolution and
//! online-friend avatar prefetch. Desktop toasts/TTS/XSOverlay/OVRT
//! notification delivery does not exist on the server.

use std::sync::{Arc, Mutex};

use vrcx_0_application::auth::AuthCredentialStore;
use vrcx_0_application_activity::notification::{
    extract_file_version, fallback_file_version, load_overlay_activity_filters,
    normalize_avatar_image_url_128, CachedNotificationUserImageResolver, NotificationConfig,
    RealtimeUserImageResolverSlot,
};
use vrcx_0_application_activity::{OverlayActivityRuntime, OverlayActivitySinkRegistry};
use vrcx_0_application_core::{
    FriendProjection, HostSessionRuntime, ImageCache, RuntimeAuthScope, RuntimeEventBus,
    TaskSupervisor,
};
use vrcx_0_application_realtime::{FriendProjectionObserver, RealtimeHostRuntime};
use vrcx_0_core::files::extract_file_id;
use vrcx_0_core::friends::StateBucket;

use crate::host_actions::RuntimeHost;
use crate::privacy_lock::PrivacyLockRuntime;

const AVATAR_PREFETCH_MAX_PATCHES: usize = 8;

pub(crate) struct ServerRuntimeServicesDeps {
    pub image_cache: Arc<ImageCache>,
    pub notification_config: Arc<dyn NotificationConfig>,
    pub auth_credentials: Arc<dyn AuthCredentialStore>,
    pub auth_scope: RuntimeAuthScope,
    pub session: HostSessionRuntime,
    pub tasks: TaskSupervisor,
    pub event_bus: RuntimeEventBus,
    pub overlay_activity: OverlayActivityRuntime,
    pub overlay_activity_sinks: OverlayActivitySinkRegistry,
}

pub struct ServerRuntimeServices {
    image_cache: Arc<ImageCache>,
    notification_config: Arc<dyn NotificationConfig>,
    session: HostSessionRuntime,
    tasks: TaskSupervisor,
    overlay_activity: OverlayActivityRuntime,
    overlay_activity_sinks: OverlayActivitySinkRegistry,
    privacy_lock: Arc<PrivacyLockRuntime>,
    pub host: RuntimeHost,
    realtime_user_image_resolver: RealtimeUserImageResolverSlot,
    realtime_user_image_resolver_owner: Mutex<Option<Arc<dyn CachedNotificationUserImageResolver>>>,
}

impl ServerRuntimeServices {
    pub(crate) fn new(deps: ServerRuntimeServicesDeps) -> vrcx_0_application_core::Result<Self> {
        let realtime_user_image_resolver = RealtimeUserImageResolverSlot::default();
        let host = RuntimeHost::new();
        deps.auth_scope
            .add_vrchat_auth_failure_observer(Arc::new(host.clone()));
        let privacy_lock = Arc::new(PrivacyLockRuntime::new(
            deps.auth_credentials,
            deps.event_bus,
        )?);
        deps.auth_scope.add_observer(privacy_lock.clone());
        Ok(Self {
            image_cache: deps.image_cache,
            notification_config: deps.notification_config,
            session: deps.session,
            tasks: deps.tasks,
            overlay_activity: deps.overlay_activity,
            overlay_activity_sinks: deps.overlay_activity_sinks,
            privacy_lock,
            host,
            realtime_user_image_resolver,
            realtime_user_image_resolver_owner: Mutex::new(None),
        })
    }

    pub fn reload_overlay_activity_filters(&self) {
        self.overlay_activity
            .set_filters(load_overlay_activity_filters(
                self.notification_config.as_ref(),
            ));
    }

    pub fn set_overlay_activity_extra_sink(
        &self,
        extra_sink: Arc<dyn vrcx_0_application_activity::OverlayActivitySink>,
    ) {
        self.overlay_activity_sinks.add(extra_sink);
    }

    pub fn set_realtime_user_image_resolver(&self, realtime_runtime: &Arc<RealtimeHostRuntime>) {
        let resolver: Arc<dyn CachedNotificationUserImageResolver> = Arc::new(
            vrcx_0_outbound_adapters::RealtimeNotificationUserImageResolver::new(realtime_runtime),
        );
        self.realtime_user_image_resolver.set(&resolver);
        match self.realtime_user_image_resolver_owner.lock() {
            Ok(mut owner) => *owner = Some(resolver),
            Err(error) => tracing::warn!(
                error = %error,
                "failed to retain realtime notification image resolver"
            ),
        }
    }

    pub fn overlay_activity(&self) -> OverlayActivityRuntime {
        self.overlay_activity.clone()
    }

    pub fn privacy_lock(&self) -> Arc<PrivacyLockRuntime> {
        Arc::clone(&self.privacy_lock)
    }

    fn prefetch_online_friend_avatars(&self, projection: &FriendProjection) {
        if projection.patches.len() > AVATAR_PREFETCH_MAX_PATCHES {
            return;
        }
        let Some(endpoint) = self
            .session
            .snapshot()
            .realtime_context
            .map(|context| context.endpoint)
            .filter(|endpoint| !endpoint.is_empty())
        else {
            return;
        };
        for patch in &projection.patches {
            if patch.presence.view.section() != StateBucket::Online {
                continue;
            }
            let user_id = patch.user_id.as_str();
            if !user_id.starts_with("usr_") {
                continue;
            }
            let Some(raw_url) = self
                .realtime_user_image_resolver
                .cached_url(&endpoint, user_id)
            else {
                continue;
            };
            let normalized = normalize_avatar_image_url_128(&raw_url, &endpoint);
            let Some(file_id) = extract_file_id(&normalized) else {
                continue;
            };
            let version = extract_file_version(&normalized, &file_id)
                .unwrap_or_else(|| fallback_file_version(&normalized));
            if version.is_empty() {
                continue;
            }
            let image_cache = Arc::clone(&self.image_cache);
            self.tasks.spawn(async move {
                let _ = image_cache.get_image(&normalized, &file_id, &version).await;
            });
        }
    }
}

impl FriendProjectionObserver for ServerRuntimeServices {
    fn on_friend_projection(&self, projection: &FriendProjection) {
        self.prefetch_online_friend_avatars(projection);
    }
}

#[cfg(test)]
mod tests;

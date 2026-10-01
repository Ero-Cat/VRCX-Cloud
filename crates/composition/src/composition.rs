use std::sync::Arc;

use vrcx_0_application_core::LocalGameContextSource;
use vrcx_0_application_realtime::FriendProjectionObserver;

use crate::{GroupOrderSource, RuntimeHostState};

pub trait RuntimeHostProfileExtension: Send + Sync {
    fn start_profile_services(&self, _state: &RuntimeHostState) {}

    fn stop_profile_services(&self) {}

    fn start_profile_maintenance(&self, _state: &RuntimeHostState) {}
}

/// Wraps the realtime transport the composition builds (e.g. a gate that
/// holds connections off while another device is responsible for data).
pub type RealtimeTransportWrapper = Arc<
    dyn Fn(
            Arc<dyn vrcx_0_application_realtime::RealtimeTransport>,
        ) -> Arc<dyn vrcx_0_application_realtime::RealtimeTransport>
        + Send
        + Sync,
>;

pub struct RuntimeHostComposition {
    pub local_game_context: Arc<dyn LocalGameContextSource>,
    pub group_order_source: Arc<dyn GroupOrderSource>,
    pub friend_projection_observer: Option<Arc<dyn FriendProjectionObserver>>,
    pub profile_extension: Option<Arc<dyn RuntimeHostProfileExtension>>,
    pub realtime_transport_wrapper: Option<RealtimeTransportWrapper>,
}

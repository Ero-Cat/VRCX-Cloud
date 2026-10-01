use vrcx_0_composition::GroupOrderSource;
use vrcx_0_host_desktop::host_capabilities::{is_host_capability_available, HostCapability};

pub struct HostGroupOrderSource;

impl GroupOrderSource for HostGroupOrderSource {
    fn read_group_order(&self, user_id: &str) -> Vec<String> {
        if !is_host_capability_available(HostCapability::RegistryPrefs) {
            return Vec::new();
        }
        vrcx_0_host_desktop::vrchat_registry::get_group_order(user_id).unwrap_or_default()
    }
}

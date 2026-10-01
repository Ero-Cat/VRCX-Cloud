use vrcx_0_i18n::OverlayMessageKey;

pub(crate) use vrcx_0_application_activity::notification::{OverlayLocale, OverlayLocalizer};

pub(crate) trait OverlayPanelLocalizer {
    fn generic_instance_location(&self) -> String;
}

impl OverlayPanelLocalizer for OverlayLocalizer {
    fn generic_instance_location(&self) -> String {
        self.label(OverlayMessageKey::OverlayGenericInstanceLocation)
    }
}

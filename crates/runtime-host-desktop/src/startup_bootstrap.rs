use vrcx_0_contracts::ConfigReadEntry;
use vrcx_0_host_desktop::host_capabilities::{current_host_capabilities, HostCapabilities};

use crate::{DesktopRuntimeHostState, Result};

#[derive(Debug, serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StartupBootstrapSnapshot {
    pub host_capabilities: HostCapabilities,
    pub config_entries: Vec<ConfigReadEntry>,
    pub system_language: String,
    pub system_culture: String,
}

impl DesktopRuntimeHostState {
    pub fn startup_bootstrap_snapshot(&self) -> Result<StartupBootstrapSnapshot> {
        Ok(StartupBootstrapSnapshot {
            host_capabilities: current_host_capabilities(),
            config_entries: self.local_data().config_list_values()?,
            system_language: system_language(),
            system_culture: system_culture(),
        })
    }
}

pub fn system_culture() -> String {
    normalized_system_locale("en-US")
}

pub fn system_language() -> String {
    normalized_system_locale("en")
}

fn normalized_system_locale(fallback: &str) -> String {
    normalize_locale(sys_locale::get_locale().unwrap_or_else(|| fallback.into()))
}

fn normalize_locale(locale: String) -> String {
    locale.replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::normalize_locale;

    #[test]
    fn locale_normalization_preserves_existing_language_tag_behavior() {
        assert_eq!(normalize_locale("zh_CN".into()), "zh-CN");
        assert_eq!(normalize_locale("en_US".into()), "en-US");
        assert_eq!(normalize_locale("zh-Hans_CN".into()), "zh-Hans-CN");
        assert_eq!(normalize_locale("en-US".into()), "en-US");
        assert_eq!(normalize_locale(String::new()), "");
    }
}

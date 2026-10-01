use crate::state::AppState;

pub(super) fn db_config_bool(state: &AppState, key: &str) -> Option<bool> {
    state.runtime_host().try_config_bool(key, false)
}

pub(super) fn app_language(state: &AppState) -> String {
    state
        .runtime_host()
        .config_string("appLanguage", "en")
        .to_ascii_lowercase()
}

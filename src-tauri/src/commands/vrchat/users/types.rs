use serde::Deserialize;
pub use vrcx_0_application::social::{
    VrchatCurrentUserBadgeInput, VrchatCurrentUserProfileUpdateInput, VrchatCurrentUserTagsInput,
    VrchatCurrentUserUpdateInput,
};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatUserInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) force: bool,
    #[serde(default)]
    pub(crate) dialog: bool,
    #[serde(default)]
    pub(crate) is_friend: Option<bool>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatUserProfileInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) as_self: bool,
}

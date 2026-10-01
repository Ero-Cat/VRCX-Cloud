use serde::Deserialize;
use vrcx_0_application::remote::{GroupSearchParams, UserSearchParams, WorldSearchParams};

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatSearchUsersInput {
    #[serde(default)]
    pub(crate) params: UserSearchParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatSearchGroupsInput {
    #[serde(default)]
    pub(crate) params: GroupSearchParams,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatSearchWorldsInput {
    #[serde(default)]
    pub(crate) params: WorldSearchParams,
    pub(crate) option: Option<String>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatSearchShortNameInput {
    #[serde(default)]
    pub(crate) short_name: String,
}

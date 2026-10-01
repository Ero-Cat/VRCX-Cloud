use serde::Deserialize;

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatFriendUserInput {
    #[serde(default)]
    pub(crate) user_id: String,
}

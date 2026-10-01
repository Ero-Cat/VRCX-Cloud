use serde::Deserialize;
use vrcx_0_application::remote::RequestInviteRequest;

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatRequestInviteSendInput {
    #[serde(default)]
    pub(crate) receiver_user_id: String,
    pub(crate) params: RequestInviteRequest,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatRequestInvitePhotoSendInput {
    #[serde(default)]
    pub(crate) receiver_user_id: String,
    pub(crate) params: RequestInviteRequest,
    #[serde(default)]
    pub(crate) image_data: String,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VrchatBoopInput {
    #[serde(default)]
    pub(crate) user_id: String,
    #[serde(default)]
    pub(crate) emoji_id: String,
    #[serde(default)]
    pub(crate) inventory_item_id: String,
}

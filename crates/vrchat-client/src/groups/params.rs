use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq, specta::Type)]
pub enum GroupMemberSort {
    #[serde(rename = "joinedAt:asc")]
    JoinedAtAsc,
    #[default]
    #[serde(rename = "joinedAt:desc")]
    JoinedAtDesc,
}

impl GroupMemberSort {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::JoinedAtAsc => "joinedAt:asc",
            Self::JoinedAtDesc => "joinedAt:desc",
        }
    }
}

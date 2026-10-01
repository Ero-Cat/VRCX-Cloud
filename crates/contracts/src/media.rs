#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UgcCategory {
    Prints,
    Stickers,
    Emoji,
}

impl UgcCategory {
    pub fn folder_name(self) -> &'static str {
        match self {
            Self::Prints => "Prints",
            Self::Stickers => "Stickers",
            Self::Emoji => "Emoji",
        }
    }
}

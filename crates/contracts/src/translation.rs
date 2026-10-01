use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum TranslationProvider {
    #[default]
    Google,
    DeepL,
    OpenAi,
}

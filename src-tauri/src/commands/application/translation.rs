#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::discovery::{TranslationResult, TranslationTranslateInput};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__translation_translate(
    state: State<'_, AppState>,
    input: TranslationTranslateInput,
) -> Result<TranslationResult, AppError> {
    state.translate(input).await
}

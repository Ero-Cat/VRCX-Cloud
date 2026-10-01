#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_contracts::FileMetadataOutput;

#[tauri::command]
#[specta::specta]
pub async fn app__file_metadata_get(
    state: State<'_, AppState>,
    file_url_or_id: String,
) -> Result<Option<FileMetadataOutput>, AppError> {
    Ok(state
        .runtime_host()
        .local_data()
        .file_metadata_get(file_url_or_id)
        .await)
}

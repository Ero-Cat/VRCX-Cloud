#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::social::{
    ModerationSyncMutationInput, ModerationSyncMutationOutput, ModerationSyncRefreshInput,
    ModerationSyncRefreshOutput,
};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__moderation_sync_refresh(
    state: State<'_, AppState>,
    input: ModerationSyncRefreshInput,
) -> Result<ModerationSyncRefreshOutput, AppError> {
    state
        .runtime_host()
        .social()
        .moderation_refresh(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__moderation_sync_update(
    state: State<'_, AppState>,
    input: ModerationSyncMutationInput,
) -> Result<ModerationSyncMutationOutput, AppError> {
    state
        .runtime_host()
        .social()
        .moderation_update(input)
        .await
        .map_err(AppError::from)
}

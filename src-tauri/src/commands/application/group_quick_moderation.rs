#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::social::{
    GroupQuickModerationActionInput, GroupQuickModerationActionOutput, GroupQuickModerationInput,
    GroupQuickModerationOutput,
};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__user_group_quick_moderation_get(
    state: State<'_, AppState>,
    input: GroupQuickModerationInput,
) -> Result<GroupQuickModerationOutput, AppError> {
    state
        .runtime_host()
        .groups()
        .quick_moderation(input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[specta::specta]
pub async fn app__user_group_quick_moderation_action(
    state: State<'_, AppState>,
    input: GroupQuickModerationActionInput,
) -> Result<GroupQuickModerationActionOutput, AppError> {
    state
        .runtime_host()
        .groups()
        .run_quick_moderation_action(input)
        .await
        .map_err(AppError::from)
}

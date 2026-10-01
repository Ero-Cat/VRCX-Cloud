#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::social::{UserGroupsOverviewInput, UserGroupsOverviewOutput};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__user_groups_overview_get(
    state: State<'_, AppState>,
    input: UserGroupsOverviewInput,
) -> Result<UserGroupsOverviewOutput, AppError> {
    state
        .runtime_host()
        .groups()
        .user_groups_overview(input)
        .await
        .map_err(AppError::from)
}

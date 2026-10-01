#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::social::{GroupCalendarInput, GroupCalendarSnapshot};

use crate::{error::AppError, state::AppState};

#[tauri::command]
#[specta::specta]
pub async fn app__group_calendar_snapshot_get(
    state: State<'_, AppState>,
    input: GroupCalendarInput,
) -> Result<GroupCalendarSnapshot, AppError> {
    state
        .runtime_host()
        .groups()
        .calendar(input)
        .await
        .map_err(AppError::from)
}

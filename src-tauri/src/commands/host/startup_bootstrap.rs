#![allow(non_snake_case)]

use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

use vrcx_0_runtime_host_desktop::StartupBootstrapSnapshot;

#[tauri::command(async)]
#[specta::specta]
pub fn app__startup_bootstrap_snapshot_get(
    state: State<'_, AppState>,
) -> Result<StartupBootstrapSnapshot, AppError> {
    Ok(state.runtime_host().startup_bootstrap_snapshot()?)
}

#![allow(non_snake_case)]

use serde_json::Value;
use tauri::State;
use vrcx_0_application::avatars::{MyAvatarByIdInput, MyAvatarsInput};

use crate::{error::AppError, state::AppState};

#[tauri::command]
#[specta::specta]
pub async fn app__my_avatars_get(
    state: State<'_, AppState>,
    input: MyAvatarsInput,
) -> Result<Vec<Value>, AppError> {
    Ok(state.runtime_host().avatars().my_avatars(input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__my_avatar_by_id_get(
    state: State<'_, AppState>,
    input: MyAvatarByIdInput,
) -> Result<Option<Value>, AppError> {
    Ok(state
        .runtime_host()
        .avatars()
        .my_avatar_by_id(input)
        .await?)
}

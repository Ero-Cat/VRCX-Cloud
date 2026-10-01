#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application_realtime::{
    SocialFavoritesBaselineInput, SocialFavoritesBaselineOutput, SocialFriendRosterBaselineInput,
    SocialFriendRosterBaselineOutput,
};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__social_baseline_refresh(
    state: State<'_, AppState>,
) -> Result<vrcx_0_application::social::SocialBaselineRefreshOutput, AppError> {
    Ok(state.runtime_host().refresh_social_baseline().await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_favorites_baseline_get(
    state: State<'_, AppState>,
    input: SocialFavoritesBaselineInput,
) -> Result<SocialFavoritesBaselineOutput, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .favorites_baseline(input)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_friend_roster_baseline_get(
    state: State<'_, AppState>,
    input: SocialFriendRosterBaselineInput,
) -> Result<SocialFriendRosterBaselineOutput, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .friend_roster_baseline(input)
        .await?)
}

#![allow(non_snake_case)]

use tauri::State;
use vrcx_0_application::social::{
    SocialFriendMutationInput, SocialFriendMutationOutcome, SocialFriendRequestAcceptInput,
    SocialFriendRequestCancelInput, SocialFriendRequestNotificationAcceptOutput,
    SocialUnfriendBatchInput, SocialUnfriendBatchResult,
};

use crate::error::AppError;
use crate::state::AppState;

#[tauri::command]
#[specta::specta]
pub async fn app__social_unfriend(
    state: State<'_, AppState>,
    input: SocialFriendMutationInput,
) -> Result<SocialFriendMutationOutcome, AppError> {
    Ok(state.runtime_host().social().unfriend(input).await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_unfriend_selection(
    state: State<'_, AppState>,
    input: SocialUnfriendBatchInput,
) -> Result<SocialUnfriendBatchResult, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .unfriend_selection(input)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_friend_request_send(
    state: State<'_, AppState>,
    input: SocialFriendMutationInput,
) -> Result<SocialFriendMutationOutcome, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .send_friend_request(input)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_friend_request_cancel(
    state: State<'_, AppState>,
    input: SocialFriendRequestCancelInput,
) -> Result<SocialFriendMutationOutcome, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .cancel_friend_request(input)
        .await?)
}

#[tauri::command]
#[specta::specta]
pub async fn app__social_friend_request_notification_accept(
    state: State<'_, AppState>,
    input: SocialFriendRequestAcceptInput,
) -> Result<SocialFriendRequestNotificationAcceptOutput, AppError> {
    Ok(state
        .runtime_host()
        .social()
        .accept_friend_request_notification(input)
        .await?)
}

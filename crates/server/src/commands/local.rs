//! Local data commands: feed queries, friend log, memos — the read model
//! the web UI renders, backed by the server SQLite kept in sync with the
//! user's desktop devices.

use std::sync::Arc;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_runtime_host_server::local_data::{
    FeedLatestQueryInput, FeedRowsQueryInput, FeedSearchQueryInput, FriendLogHistoryQueryInput,
};
use vrcx_0_runtime_host_server::ServerRuntimeHostState;

use super::{ok, run_blocking};

fn arg<T: serde::de::DeserializeOwned>(args: &serde_json::Value, key: &str) -> Result<T, ApiError> {
    let value = args.get(key).cloned().unwrap_or(serde_json::Value::Null);
    serde_json::from_value(value)
        .map_err(|error| ApiError::BadRequest(format!("invalid `{key}` argument: {error}")))
}

pub fn register(registry: &mut CommandRegistry) {
    registry.register(
        "app__feed_latest_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: FeedLatestQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("feed latest query", move || {
                local_data
                    .query_feed_latest(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__feed_search_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: FeedSearchQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("feed search query", move || {
                local_data
                    .query_feed_search(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__feed_rows_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: FeedRowsQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("feed rows query", move || {
                local_data
                    .feed_rows_query(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__friend_log_current_list",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("friend log current", move || {
                local_data
                    .friend_log_current_list(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__friend_log_history_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: FriendLogHistoryQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("friend log history", move || {
                local_data
                    .friend_log_history_query(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__memo_list_users",
        |state: Arc<ServerRuntimeHostState>, _args| async move {
            let local_data = state.local_data().clone();
            run_blocking("memo list users", move || {
                local_data
                    .memo_list_users()
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__memo_get_user",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let local_data = state.local_data().clone();
            run_blocking("memo get user", move || {
                local_data
                    .memo_get_user(user_id)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__memo_save_user",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let user_id: String = arg(&args, "userId")?;
            let memo: String = arg(&args, "memo")?;
            let local_data = state.local_data().clone();
            run_blocking("memo save user", move || {
                local_data
                    .memo_save_user(user_id, memo)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
}

//! Social analytics commands: bio history (diff view source), status light
//! distribution, and the mutual instance query — read-models derived from
//! the synced feed event tables.

use std::sync::Arc;

use crate::transport::error::ApiError;
use crate::transport::invoke::CommandRegistry;
use vrcx_0_runtime_host_server::local_data::{
    FeedBioHistoryQueryInput, RelationshipTimelineRowsInput, StatusStatsViewInput,
    TwoPersonRelationshipQueryInput,
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
        "app__feed_bio_history_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let query: FeedBioHistoryQueryInput = arg(&args, "query")?;
            let local_data = state.local_data().clone();
            run_blocking("feed bio history", move || {
                local_data
                    .feed_bio_history_query(query)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__status_stats_view",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: StatusStatsViewInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("status stats view", move || {
                local_data
                    .status_stats_view(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__two_person_relationship_query",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: TwoPersonRelationshipQueryInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("two person relationship query", move || {
                local_data
                    .two_person_relationship_query(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
    registry.register(
        "app__relationship_timeline_rows",
        |state: Arc<ServerRuntimeHostState>, args| async move {
            let input: RelationshipTimelineRowsInput = arg(&args, "input")?;
            let local_data = state.local_data().clone();
            run_blocking("relationship timeline rows", move || {
                local_data
                    .relationship_timeline_rows(input)
                    .map_err(vrcx_0_composition::Error::from)
            })
            .await
            .and_then(ok)
        },
    );
}

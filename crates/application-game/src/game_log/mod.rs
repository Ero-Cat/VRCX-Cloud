//! Game-log data model and queries.
//!
//! The server consumes game-log data through the remote sync protocol;
//! local ingest (log-file watcher, parser, processor) is desktop-only and
//! intentionally absent from this fork.

pub(crate) mod runtime_state;

pub use instance_history::{
    instance_history_query, InstanceHistoryEntryOutput, InstanceHistoryQueryInput,
};
pub use runtime_state::{duration_ms, world_id_from_location, PlayerState, RuntimeSnapshot};
pub use sessions_view::{
    game_log_sessions_query, GameLogSessionDto, GameLogSessionEventDto, GameLogSessionMemberDto,
    GameLogSessionPlayerDurationRowDto, GameLogSessionsQueryInput,
};

mod instance_history;
mod sessions_view;

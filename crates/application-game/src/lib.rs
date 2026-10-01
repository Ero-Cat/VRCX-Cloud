pub use vrcx_0_application_core::{Error, Result};

mod background_capabilities;
mod game_event_bus;
mod game_log;
mod ports;

pub use background_capabilities::{
    build_background_presence_facts, presence_automation_rule_enabled_set,
    presence_automation_rules_get, presence_automation_rules_set,
    run_background_presence_automation, BackgroundPresenceAutomationResult,
    BackgroundPresenceAutomationState, BackgroundPresenceFacts, BackgroundPresenceFactsInput,
    PresenceAutomationRuleKind,
};
pub use game_event_bus::NowPlayingSnapshot;
pub use game_log::{
    duration_ms, game_log_sessions_query, instance_history_query, world_id_from_location,
    GameLogSessionDto, GameLogSessionEventDto, GameLogSessionMemberDto,
    GameLogSessionPlayerDurationRowDto, GameLogSessionsQueryInput, InstanceHistoryEntryOutput,
    InstanceHistoryQueryInput, PlayerState, RuntimeSnapshot,
};
pub use ports::{BackgroundRemoteApi, GameStateStore};

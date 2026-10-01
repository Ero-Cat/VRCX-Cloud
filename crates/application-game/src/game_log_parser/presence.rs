use vrcx_0_core::game_log_parser::{
    clean_location, parse_room_log_event, GameLogEventKind, RoomLogEvent,
};

use super::context::LogContext;
use super::sink::GameLogParseSink;

pub(super) struct ParsedUserInfo {
    pub(super) display_name: String,
    pub(super) user_id: String,
}

pub(super) fn parse_user_info(s: &str) -> ParsedUserInfo {
    if let Some(pos) = s.rfind(" (") {
        let display_name = s[..pos].to_string();
        let end = s
            .rfind(')')
            .filter(|end| *end >= pos + 2)
            .unwrap_or(s.len());
        let user_id: String = s[pos + 2..end]
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '~' | ':' | '(' | ')'))
            .collect();
        ParsedUserInfo {
            display_name,
            user_id,
        }
    } else {
        ParsedUserInfo {
            display_name: s.to_string(),
            user_id: String::new(),
        }
    }
}

pub(super) fn parse_location(
    out: &mut dyn GameLogParseSink,
    fname: &str,
    line: &str,
    content: &str,
    ctx: &mut LogContext,
) -> bool {
    match parse_room_log_event(line, content) {
        Some(RoomLogEvent::Entering { world_name }) => {
            ctx.recent_world_name = world_name.to_string();
            true
        }
        Some(RoomLogEvent::Joining { location }) => {
            let location = clean_location(location);
            out.push_event(
                fname,
                line,
                GameLogEventKind::Location {
                    location,
                    world_name: ctx.recent_world_name.clone(),
                },
            );
            ctx.last_audio_device.clear();
            ctx.video_errors.clear();
            out.set_vrc_closed_gracefully(false);
            true
        }
        Some(RoomLogEvent::Left) | None => false,
    }
}

pub(super) fn parse_location_destination(
    out: &mut dyn GameLogParseSink,
    fname: &str,
    line: &str,
    content: &str,
    ctx: &mut LogContext,
) -> bool {
    if parse_room_log_event(line, content) == Some(RoomLogEvent::Left) {
        out.push_event(
            fname,
            line,
            GameLogEventKind::LocationDestination {
                location: ctx.location_destination.clone(),
            },
        );
        ctx.location_destination.clear();
        return true;
    }

    if content.contains("[Behaviour] Destination fetching: ") {
        if let Some(pos) = line.rfind("] Destination fetching: ") {
            ctx.location_destination = clean_location(&line[pos + 24..]);
        }
        return true;
    }

    false
}

pub(super) fn parse_player_joined_or_left(
    out: &mut dyn GameLogParseSink,
    fname: &str,
    line: &str,
    content: &str,
) -> bool {
    if content.contains("[Behaviour] OnPlayerJoined") && !content.contains("] OnPlayerJoined:") {
        if let Some(pos) = line.rfind("] OnPlayerJoined") {
            let user_info = line.get(pos + 17..).unwrap_or_default();
            let ParsedUserInfo {
                display_name,
                user_id,
            } = parse_user_info(user_info);
            if !display_name.is_empty() || !user_id.is_empty() {
                out.push_event(
                    fname,
                    line,
                    GameLogEventKind::PlayerJoined {
                        display_name,
                        user_id,
                    },
                );
            }
        }
        return true;
    }

    if content.contains("[Behaviour] OnPlayerLeft")
        && !content.contains("] OnPlayerLeftRoom")
        && !content.contains("] OnPlayerLeft:")
    {
        if let Some(pos) = line.rfind("] OnPlayerLeft") {
            let user_info = line.get(pos + 15..).unwrap_or_default();
            let ParsedUserInfo {
                display_name,
                user_id,
            } = parse_user_info(user_info);
            if !display_name.is_empty() || !user_id.is_empty() {
                out.push_event(
                    fname,
                    line,
                    GameLogEventKind::PlayerLeft {
                        display_name,
                        user_id,
                    },
                );
            }
        }
        return true;
    }

    false
}

pub(super) fn parse_portal_spawn(out: &mut dyn GameLogParseSink, fname: &str, line: &str) -> bool {
    if line.contains("[Behaviour] Instantiated a (Clone [")
        && line.contains("] Portals/PortalInternalDynamic)")
    {
        out.push_event(fname, line, GameLogEventKind::PortalSpawn);
        return true;
    }
    false
}

pub(super) fn parse_notification(
    out: &mut dyn GameLogParseSink,
    fname: &str,
    line: &str,
    content: &str,
) -> bool {
    if !content.starts_with("[API] Received Notification: <") {
        return false;
    }
    if let Some(pos) = line.rfind("> received at ") {
        if let Some(start) = line.find("[API] Received Notification: <") {
            let data = &line[start + 30..pos];
            out.push_event(
                fname,
                line,
                GameLogEventKind::Notification { data: data.into() },
            );
        }
    }
    true
}

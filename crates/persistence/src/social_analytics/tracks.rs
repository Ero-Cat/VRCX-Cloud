//! Shared interval helpers for the social analytics read-models: time
//! parsing plus presence-interval derivation from the append-only feed
//! tables and the owner's own game log.

use std::collections::HashMap;

use serde_json::Value;

use crate::common::row_string;
use crate::database::DatabaseService;
use crate::Error;

pub(crate) const MAX_SEGMENT_MS: i64 = 24 * 60 * 60 * 1000;
pub(crate) const PRESENCE_FALLBACK_GAP_MS: i64 = 15 * 60 * 1000;

pub(crate) fn parse_time_ms(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|value| value.timestamp_millis())
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|value| value.and_utc().timestamp_millis())
        })
}

pub(crate) fn iso_from_ms(ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
        .unwrap_or_else(chrono::Utc::now)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Presence intervals (online windows) for one friend, from Online/Offline
/// pairs; falls back to location-change clustering when no pairs exist
/// (mirrors the activity view fallback for continuously-online friends).
pub(crate) fn presence_intervals_for_user(
    db: &DatabaseService,
    user_prefix: &str,
    user_id: &str,
    from_iso: &str,
    now_ms: i64,
) -> Result<Vec<(i64, i64)>, Error> {
    let mut params = HashMap::new();
    params.insert("@user_id".into(), Value::String(user_id.to_string()));
    params.insert("@from_iso".into(), Value::String(from_iso.to_string()));
    let events: Vec<(i64, bool)> = db
        .execute(
            &format!(
                "SELECT created_at, type FROM {user_prefix}_feed_online_offline
                 WHERE user_id = @user_id AND created_at >= @from_iso
                 ORDER BY created_at, id"
            ),
            &params,
        )?
        .into_iter()
        .filter_map(|row| {
            let created_at = row_string(&row, 0);
            let is_online = row_string(&row, 1) == "Online";
            parse_time_ms(&created_at).map(|at_ms| (at_ms, is_online))
        })
        .collect();

    let mut intervals = Vec::new();
    let mut current_start: Option<i64> = None;
    for (at_ms, is_online) in events {
        if is_online {
            current_start = Some(at_ms);
        } else if let Some(start) = current_start.take() {
            if at_ms > start {
                intervals.push((start, at_ms));
            }
        }
    }
    if let Some(start) = current_start {
        intervals.push((start, now_ms.max(start)));
    }
    if !intervals.is_empty() {
        return Ok(cap_intervals(intervals, now_ms));
    }

    // Fallback: cluster gps timestamps; each cluster is one approximate
    // online window.
    let stamps: Vec<i64> = db
        .execute(
            &format!(
                "SELECT created_at FROM {user_prefix}_feed_gps
                 WHERE user_id = @user_id AND created_at >= @from_iso
                 ORDER BY created_at"
            ),
            &params,
        )?
        .into_iter()
        .filter_map(|row| parse_time_ms(&row_string(&row, 0)))
        .collect();
    let mut intervals = Vec::new();
    let mut cluster: Option<(i64, i64)> = None;
    for stamp in stamps {
        match cluster {
            Some((start, last)) if stamp - last <= PRESENCE_FALLBACK_GAP_MS => {
                cluster = Some((start, stamp));
            }
            Some((start, last)) => {
                intervals.push((start, last));
                cluster = Some((stamp, stamp));
            }
            None => cluster = Some((stamp, stamp)),
        }
    }
    if let Some((start, last)) = cluster {
        intervals.push((start, last));
    }
    Ok(cap_intervals(intervals, now_ms))
}

/// The owner's own online windows from the desktop-recorded game-log
/// location rows; empty on server-only deployments.
pub(crate) fn self_presence_intervals_from_game_log(
    db: &DatabaseService,
    owner_user_id: &str,
    from_iso: &str,
    now_ms: i64,
) -> Result<Vec<(i64, i64)>, Error> {
    crate::game_log::ensure_game_log_tables(db)?;
    let owner_row_id = crate::ownership::owner_id_for_filter(
        db,
        &vrcx_0_core::OwnerId::new(owner_user_id.to_string()),
    )?;
    let mut params = HashMap::new();
    params.insert("@owner_id".into(), Value::from(owner_row_id.value()));
    params.insert("@from".into(), Value::String(from_iso.to_string()));
    let stamps: Vec<i64> = db
        .execute(
            "SELECT created_at FROM gamelog_location
             WHERE owner_id IN (0, @owner_id) AND created_at >= @from
             ORDER BY created_at, id",
            &params,
        )?
        .into_iter()
        .filter_map(|row| parse_time_ms(&row_string(&row, 0)))
        .collect();
    let mut intervals: Vec<(i64, i64)> = Vec::new();
    let mut cluster: Option<(i64, i64)> = None;
    for stamp in stamps {
        match cluster {
            Some((start, last)) if stamp - last <= PRESENCE_FALLBACK_GAP_MS => {
                cluster = Some((start, stamp));
            }
            Some((start, last)) => {
                intervals.push((start, last));
                cluster = Some((stamp, stamp));
            }
            None => cluster = Some((stamp, stamp)),
        }
    }
    if let Some((start, last)) = cluster {
        intervals.push((start, last));
    }
    Ok(cap_intervals(intervals, now_ms))
}

fn cap_intervals(intervals: Vec<(i64, i64)>, now_ms: i64) -> Vec<(i64, i64)> {
    intervals
        .into_iter()
        .filter_map(|(start, end)| {
            let end = end.min(now_ms).min(start + MAX_SEGMENT_MS);
            (end > start).then_some((start, end))
        })
        .collect()
}

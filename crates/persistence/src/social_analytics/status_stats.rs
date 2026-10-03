//! Status-light distribution, VRCX-jirai parity: status intervals run from
//! each status change to the next one (the last one to "now"), intersected
//! with the Online/Offline presence windows. When the friend has no
//! presence rows at all the status intervals count in full — a status
//! record implies they were online at that moment. No GPS fallback:
//! cluster-approximated presence is what made the numbers diverge from the
//! recorded events.

use std::collections::HashMap;

use serde_json::Value;

use vrcx_0_core::friends::normalize_user_status;

use crate::common::{normalize_text, row_string};
use crate::database::DatabaseService;
use crate::realtime::{ensure_realtime_tables, normalize_user_table_prefix};
use crate::Error;

use super::tracks::{iso_from_ms, parse_time_ms, MAX_SEGMENT_MS};
use vrcx_0_contracts::social_analytics::{
    StatusStatsDailyBucket, StatusStatsTotal, StatusStatsViewInput, StatusStatsViewOutput,
};

const DEFAULT_RANGE_DAYS: i64 = 90;
const MAX_RANGE_DAYS: i64 = 365;
const DAY_MS: i64 = 86_400_000;
const MIN_MS: i64 = 60_000;

pub fn status_stats_view(
    db: &DatabaseService,
    input: StatusStatsViewInput,
) -> Result<StatusStatsViewOutput, Error> {
    let owner_user_id = normalize_text(&input.owner_user_id);
    let target_user_id = normalize_text(&input.target_user_id);
    if owner_user_id.is_empty() || target_user_id.is_empty() {
        return Ok(StatusStatsViewOutput::default());
    }
    let now_ms = if input.now_ms > 0 {
        input.now_ms
    } else {
        chrono::Utc::now().timestamp_millis()
    };
    let range_days = if input.range_days > 0 {
        input.range_days.min(MAX_RANGE_DAYS)
    } else {
        DEFAULT_RANGE_DAYS
    };
    let from_iso = iso_from_ms(now_ms - range_days * DAY_MS);
    let user_prefix = normalize_user_table_prefix(&owner_user_id)?;
    ensure_realtime_tables(db, &user_prefix)?;

    let mut params = HashMap::new();
    params.insert("@user_id".into(), Value::String(target_user_id.clone()));
    params.insert("@from_iso".into(), Value::String(from_iso.clone()));

    // Status change rows → [created_at, next created_at] intervals, the
    // last one open-ended to now.
    let status_events: Vec<(i64, String)> = db
        .execute(
            &format!(
                "SELECT created_at, status FROM {user_prefix}_feed_status
                 WHERE user_id = @user_id AND created_at >= @from_iso
                 ORDER BY created_at, id"
            ),
            &params,
        )?
        .into_iter()
        .filter_map(|row| {
            let created_at = row_string(&row, 0);
            let status = normalize_user_status(&row_string(&row, 1));
            parse_time_ms(&created_at).map(|at_ms| (at_ms, status))
        })
        .filter(|(_, status)| !status.is_empty())
        .collect();
    if status_events.is_empty() {
        return Ok(StatusStatsViewOutput {
            range_days,
            totals: Vec::new(),
            daily: Vec::new(),
            tracked_minutes: 0,
            has_any_data: false,
            built_at: iso_from_ms(now_ms),
        });
    }
    let status_intervals: Vec<(i64, i64, String)> = status_events
        .windows(2)
        .map(|pair| (pair[0].0, pair[1].0, pair[0].1.clone()))
        .chain(std::iter::once((
            status_events[status_events.len() - 1].0,
            now_ms,
            status_events[status_events.len() - 1].1.clone(),
        )))
        .filter(|(start, end, _)| end > start)
        .collect();

    // Presence windows: Online → next Offline (a following Online also
    // closes the window, mirroring jirai's missed-Offline handling).
    let online_events: Vec<(i64, bool)> = db
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
            let kind = row_string(&row, 1);
            let is_online = kind == "Online";
            (is_online || kind == "Offline")
                .then(|| parse_time_ms(&created_at).map(|at_ms| (at_ms, is_online)))
                .flatten()
        })
        .collect();
    let mut online_intervals: Vec<(i64, i64)> = Vec::new();
    let mut index = 0;
    while index < online_events.len() {
        if online_events[index].1 {
            let start = online_events[index].0;
            let mut end = now_ms;
            for follower in online_events.iter().skip(index + 1) {
                // Any subsequent presence record closes the window.
                end = follower.0;
                if !follower.1 {
                    break;
                }
            }
            if end > start {
                online_intervals.push((start, end));
            }
        }
        index += 1;
    }

    // No presence rows at all → count the status intervals in full.
    let active_intervals: Vec<(i64, i64, String)> = if online_intervals.is_empty() {
        status_intervals.clone()
    } else {
        let mut active = Vec::new();
        for (start, end, status) in &status_intervals {
            for (online_start, online_end) in &online_intervals {
                if *online_start >= *end {
                    break;
                }
                if *online_end <= *start {
                    continue;
                }
                let from = (*start).max(*online_start);
                let to = (*end).min(*online_end);
                if to > from {
                    active.push((from, to, status.clone()));
                }
            }
        }
        active
    };

    let mut totals: HashMap<String, i64> = HashMap::new();
    let mut daily: HashMap<(String, String), i64> = HashMap::new();
    let mut tracked_ms: i64 = 0;
    let offset_ms = input.utc_offset_minutes * MIN_MS;
    for (start, end, status) in active_intervals {
        let end = end.min(start + MAX_SEGMENT_MS);
        if end <= start || status == "offline" {
            continue;
        }
        tracked_ms += end - start;
        *totals.entry(status.clone()).or_insert(0) += end - start;
        let mut cursor = start;
        while cursor < end {
            let local_day_start = ((cursor + offset_ms) / DAY_MS) * DAY_MS;
            let next_boundary = (local_day_start + DAY_MS - offset_ms).max(cursor + 1);
            let chunk_end = end.min(next_boundary);
            if chunk_end > cursor {
                let day = local_day_label(cursor + offset_ms);
                *daily.entry((day, status.clone())).or_insert(0) += chunk_end - cursor;
            }
            cursor = chunk_end;
        }
    }

    let tracked_minutes = tracked_ms / MIN_MS;
    let mut totals: Vec<StatusStatsTotal> = totals
        .into_iter()
        .map(|(status, ms)| StatusStatsTotal {
            status,
            minutes: ms / MIN_MS,
            share: 0.0,
        })
        .collect();
    for total in &mut totals {
        total.share = if tracked_minutes > 0 {
            (total.minutes as f64 / tracked_minutes as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    totals.sort_by(|a, b| b.minutes.cmp(&a.minutes).then(a.status.cmp(&b.status)));
    let mut daily: Vec<StatusStatsDailyBucket> = daily
        .into_iter()
        .map(|((day, status), ms)| StatusStatsDailyBucket {
            day,
            status,
            minutes: ms / MIN_MS,
        })
        .collect();
    daily.sort_by(|a, b| a.day.cmp(&b.day).then(a.status.cmp(&b.status)));
    let has_any_data = !totals.is_empty();
    Ok(StatusStatsViewOutput {
        range_days,
        totals,
        daily,
        tracked_minutes,
        has_any_data,
        built_at: iso_from_ms(now_ms),
    })
}

fn local_day_label(local_ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(local_ms)
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%d")
        .to_string()
}

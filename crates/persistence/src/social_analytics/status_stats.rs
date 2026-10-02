use std::collections::HashMap;

use serde_json::Value;

use vrcx_0_core::friends::normalize_user_status;

use crate::common::{normalize_text, row_string};
use crate::database::DatabaseService;
use crate::realtime::{ensure_realtime_tables, normalize_user_table_prefix};
use crate::Error;

use super::tracks::{
    iso_from_ms, parse_time_ms, presence_intervals_for_user, self_presence_intervals_from_game_log,
    MAX_SEGMENT_MS,
};
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
    let from_ms = now_ms - range_days * DAY_MS;
    let from_iso = iso_from_ms(from_ms);
    let user_prefix = normalize_user_table_prefix(&owner_user_id)?;
    ensure_realtime_tables(db, &user_prefix)?;

    let presence = if input.is_self {
        self_presence_intervals_from_game_log(db, &owner_user_id, &from_iso, now_ms)?
    } else {
        presence_intervals_for_user(db, &user_prefix, &target_user_id, &from_iso, now_ms)?
    };

    let mut params = HashMap::new();
    params.insert("@user_id".into(), Value::String(target_user_id.clone()));
    params.insert("@from_iso".into(), Value::String(from_iso.clone()));
    let status_events: Vec<(i64, String)> = if input.is_self {
        db.execute(
            &format!(
                "SELECT created_at, value FROM {user_prefix}_self_profile_log
                 WHERE field = 'status' AND created_at >= @from_iso
                 ORDER BY created_at, id"
            ),
            &params,
        )?
        .into_iter()
        .filter_map(|row| {
            let created_at = row_string(&row, 0);
            let value = row_string(&row, 1);
            parse_time_ms(&created_at).map(|at_ms| (at_ms, value))
        })
        .collect()
    } else {
        db.execute(
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
            let status = row_string(&row, 1);
            parse_time_ms(&created_at).map(|at_ms| (at_ms, status))
        })
        .collect()
    };
    let status_events: Vec<(i64, String)> = status_events
        .into_iter()
        .filter_map(|(at_ms, status)| {
            let status = normalize_user_status(&status);
            (!status.is_empty()).then_some((at_ms, status))
        })
        .collect();

    let mut totals: HashMap<String, i64> = HashMap::new();
    let mut daily: HashMap<(String, String), i64> = HashMap::new();
    let mut tracked_ms: i64 = 0;

    for (start_ms, end_ms) in presence {
        // Status at the start of the window: the latest known event at or
        // before it; events during the window split it further.
        let mut split_points = Vec::new();
        for (at_ms, _) in &status_events {
            if *at_ms > start_ms && *at_ms < end_ms {
                split_points.push(*at_ms);
            }
        }
        let mut window_start = start_ms;
        let mut current_status: String = match status_events
            .partition_point(|(at_ms, _)| *at_ms <= start_ms)
            .checked_sub(1)
            .map(|index| status_events[index].1.clone())
        {
            Some(status) => status,
            None => {
                // No status known at the window start: begin tracking at
                // the first event inside the window instead of discarding
                // the whole window.
                match split_points.first() {
                    Some(first_at) => {
                        window_start = *first_at;
                        status_at(&status_events, *first_at).unwrap_or_default()
                    }
                    None => continue,
                }
            }
        };
        if current_status.is_empty() {
            continue;
        }
        for split_at in split_points {
            if split_at <= window_start {
                continue;
            }
            if let Some(status) = status_at(&status_events, split_at) {
                accumulate_interval(
                    &mut totals,
                    &mut daily,
                    &mut tracked_ms,
                    window_start,
                    split_at,
                    &current_status,
                    input.utc_offset_minutes,
                );
                current_status = status;
            }
            window_start = split_at;
        }
        accumulate_interval(
            &mut totals,
            &mut daily,
            &mut tracked_ms,
            window_start,
            end_ms,
            &current_status,
            input.utc_offset_minutes,
        );
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

fn status_at(events: &[(i64, String)], at_ms: i64) -> Option<String> {
    events
        .iter()
        .rev()
        .find(|(event_at, _)| *event_at == at_ms)
        .map(|(_, status)| status.clone())
}

fn accumulate_interval(
    totals: &mut HashMap<String, i64>,
    daily: &mut HashMap<(String, String), i64>,
    tracked_ms: &mut i64,
    start_ms: i64,
    end_ms: i64,
    status: &str,
    utc_offset_minutes: i64,
) {
    let end_ms = end_ms.min(start_ms + MAX_SEGMENT_MS);
    if end_ms <= start_ms || status == "offline" {
        return;
    }
    *tracked_ms += end_ms - start_ms;
    *totals.entry(status.to_string()).or_insert(0) += end_ms - start_ms;
    // Split the interval across local day boundaries.
    let offset_ms = utc_offset_minutes * MIN_MS;
    let mut cursor = start_ms;
    while cursor < end_ms {
        let local_day_start = ((cursor + offset_ms) / DAY_MS) * DAY_MS;
        let next_boundary = (local_day_start + DAY_MS - offset_ms).max(cursor + 1);
        let chunk_end = end_ms.min(next_boundary);
        if chunk_end > cursor {
            let day = local_day_label(cursor + offset_ms);
            *daily.entry((day, status.to_string())).or_insert(0) += chunk_end - cursor;
        }
        cursor = chunk_end;
    }
}

fn local_day_label(local_ms: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp_millis(local_ms)
        .unwrap_or_else(chrono::Utc::now)
        .format("%Y-%m-%d")
        .to_string()
}

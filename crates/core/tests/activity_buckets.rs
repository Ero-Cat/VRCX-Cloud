use chrono::DateTime;
use vrcx_0_core::activity_buckets::{
    activity_streaks, activity_timeline, utc_offset_label, ActivityTimeBucket,
};

fn ms(value: &str) -> i64 {
    DateTime::parse_from_rfc3339(value)
        .unwrap()
        .timestamp_millis()
}

fn row_minutes(rows: &[vrcx_0_core::activity_buckets::ActivityBucket], key: &str) -> Option<i64> {
    rows.iter()
        .find(|row| row.key == key)
        .map(|row| row.minutes)
}

#[test]
fn timeline_month_splits_boundary() {
    let rows = activity_timeline(
        &[(ms("2025-01-31T23:30:00Z"), ms("2025-02-01T00:30:00Z"))],
        ActivityTimeBucket::Month,
        0,
        None,
        None,
    );

    assert_eq!(row_minutes(&rows, "2025-01"), Some(30));
    assert_eq!(row_minutes(&rows, "2025-02"), Some(30));
}

#[test]
fn timeline_month_splits_partial_minute_at_boundary() {
    let rows = activity_timeline(
        &[(ms("2025-01-31T23:59:30Z"), ms("2025-02-01T00:01:30Z"))],
        ActivityTimeBucket::Month,
        0,
        None,
        None,
    );

    assert_eq!(row_minutes(&rows, "2025-01"), Some(0));
    assert_eq!(row_minutes(&rows, "2025-02"), Some(1));
}

#[test]
fn timeline_hour_local_offset() {
    let rows = activity_timeline(
        &[(ms("2025-01-01T18:00:00Z"), ms("2025-01-01T20:00:00Z"))],
        ActivityTimeBucket::HourOfDay,
        540,
        None,
        None,
    );

    assert_eq!(row_minutes(&rows, "03"), Some(60));
    assert_eq!(row_minutes(&rows, "04"), Some(60));
}

#[test]
fn timeline_month_offset_shifts_day() {
    let rows = activity_timeline(
        &[(ms("2025-02-28T20:00:00Z"), ms("2025-02-28T21:00:00Z"))],
        ActivityTimeBucket::Month,
        540,
        None,
        None,
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, "2025-03");
    assert_eq!(rows[0].minutes, 60);
}

#[test]
fn timeline_histogram_fills_all_hours() {
    let rows = activity_timeline(
        &[(ms("2025-01-01T03:00:00Z"), ms("2025-01-01T04:00:00Z"))],
        ActivityTimeBucket::HourOfDay,
        0,
        None,
        None,
    );

    assert_eq!(rows.len(), 24);
    assert_eq!(row_minutes(&rows, "03"), Some(60));
    assert_eq!(row_minutes(&rows, "04"), Some(0));
}

#[test]
fn timeline_weekday_fills_all_days() {
    let rows = activity_timeline(
        &[(ms("2025-01-06T03:00:00Z"), ms("2025-01-06T04:00:00Z"))],
        ActivityTimeBucket::DayOfWeek,
        0,
        None,
        None,
    );

    assert_eq!(rows.len(), 7);
    assert_eq!(rows[0].key, "0");
    assert_eq!(rows[0].label, "Monday");
    assert_eq!(rows[0].minutes, 60);
}

#[test]
fn timeline_window_clamps() {
    let rows = activity_timeline(
        &[
            (ms("2025-01-01T00:00:00Z"), ms("2025-01-01T01:00:00Z")),
            (ms("2025-02-01T00:00:00Z"), ms("2025-02-01T01:00:00Z")),
        ],
        ActivityTimeBucket::Month,
        0,
        Some(ms("2025-02-01T00:00:00Z")),
        Some(ms("2025-02-02T00:00:00Z")),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, "2025-02");
    assert_eq!(rows[0].minutes, 60);
}

#[test]
fn timeline_week_uses_iso_week_across_year() {
    let rows = activity_timeline(
        &[(ms("2024-12-30T00:00:00Z"), ms("2024-12-30T01:00:00Z"))],
        ActivityTimeBucket::Week,
        0,
        None,
        None,
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, "2025-W01");
    assert_eq!(rows[0].label, "2025-W01");
    assert_eq!(rows[0].minutes, 60);
}

#[test]
fn timeline_empty() {
    assert!(activity_timeline(&[], ActivityTimeBucket::Month, 0, None, None).is_empty());
}

#[test]
fn streaks_consecutive_days() {
    let now = ms("2025-01-04T00:00:00Z");
    let streaks = activity_streaks(
        &[
            (ms("2025-01-01T01:00:00Z"), ms("2025-01-01T02:00:00Z")),
            (ms("2025-01-02T01:00:00Z"), ms("2025-01-02T02:00:00Z")),
            (ms("2025-01-03T01:00:00Z"), ms("2025-01-03T02:00:00Z")),
        ],
        now,
        0,
    );

    assert_eq!(streaks.longest_play_streak_days, 3);
    assert_eq!(streaks.longest_break_days, 0);
    assert_eq!(streaks.total_active_days, 3);
}

#[test]
fn streaks_gap() {
    let streaks = activity_streaks(
        &[
            (ms("2025-01-01T01:00:00Z"), ms("2025-01-01T02:00:00Z")),
            (ms("2025-01-06T01:00:00Z"), ms("2025-01-06T02:00:00Z")),
        ],
        ms("2025-01-06T03:00:00Z"),
        0,
    );

    assert_eq!(streaks.longest_break_days, 4);
    assert_eq!(streaks.longest_play_streak_days, 1);
    assert_eq!(streaks.total_active_days, 2);
}

#[test]
fn streaks_current_break() {
    let streaks = activity_streaks(
        &[(ms("2025-01-01T01:00:00Z"), ms("2025-01-01T02:00:00Z"))],
        ms("2025-01-04T03:00:00Z"),
        0,
    );

    assert_eq!(streaks.current_break_days, 3);
}

#[test]
fn streaks_open_tail_today() {
    let now = ms("2025-01-04T03:00:00Z");
    let streaks = activity_streaks(&[(ms("2025-01-04T01:00:00Z"), now)], now, 0);

    assert_eq!(streaks.current_break_days, 0);
}

#[test]
fn streaks_crosses_midnight() {
    let streaks = activity_streaks(
        &[(ms("2025-01-01T23:30:00Z"), ms("2025-01-02T00:30:00Z"))],
        ms("2025-01-02T01:00:00Z"),
        0,
    );

    assert_eq!(streaks.total_active_days, 2);
    assert_eq!(streaks.longest_play_streak_days, 2);
}

#[test]
fn streaks_midnight_end_is_exclusive() {
    let streaks = activity_streaks(
        &[(ms("2025-01-01T23:00:00Z"), ms("2025-01-02T00:00:00Z"))],
        ms("2025-01-02T12:00:00Z"),
        0,
    );

    assert_eq!(streaks.total_active_days, 1);
    assert_eq!(streaks.current_break_days, 1);
}

#[test]
fn streaks_empty() {
    let streaks = activity_streaks(&[], ms("2025-01-01T00:00:00Z"), 0);

    assert_eq!(streaks.total_minutes, 0);
    assert_eq!(streaks.session_count, 0);
    assert_eq!(streaks.first_session_ms, None);
    assert_eq!(streaks.last_session_ms, None);
}

#[test]
fn offset_label_formats_utc_offsets() {
    assert_eq!(utc_offset_label(0), "UTC");
    assert_eq!(utc_offset_label(540), "UTC+09:00");
    assert_eq!(utc_offset_label(-300), "UTC-05:00");
}

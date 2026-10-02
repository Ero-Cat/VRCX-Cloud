use std::collections::HashMap;

use serde_json::Value;

use crate::common::{normalize_text, row_string};
use crate::database::DatabaseService;
use crate::realtime::{ensure_realtime_tables, normalize_user_table_prefix};
use crate::Error;

use super::tracks::parse_time_ms;
use vrcx_0_contracts::social_analytics::{
    FeedBioHistoryOutput, FeedBioHistoryQueryInput, FeedBioHistoryRow,
};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 200;
const DEFAULT_MERGE_HOURS: i64 = 24;

pub fn feed_bio_history_query(
    db: &DatabaseService,
    query: FeedBioHistoryQueryInput,
) -> Result<FeedBioHistoryOutput, Error> {
    let user_id = normalize_text(&query.user_id);
    let target_user_id = normalize_text(&query.target_user_id);
    if user_id.is_empty() || target_user_id.is_empty() {
        return Ok(FeedBioHistoryOutput::default());
    }
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_realtime_tables(db, &user_prefix)?;

    let mut params = HashMap::new();
    params.insert("@user_id".into(), Value::String(target_user_id));
    let mut clauses = vec!["user_id = @user_id".to_string()];
    if !query.date_from.trim().is_empty() {
        clauses.push("julianday(created_at) >= julianday(@date_from)".into());
        params.insert("@date_from".into(), Value::String(query.date_from.clone()));
    }
    if !query.date_to.trim().is_empty() {
        clauses.push("julianday(created_at) <= julianday(@date_to)".into());
        params.insert("@date_to".into(), Value::String(query.date_to.clone()));
    }
    let rows = db.execute(
        &format!(
            "SELECT id, created_at, display_name, previous_bio, bio FROM {user_prefix}_feed_bio
             WHERE {} ORDER BY created_at, id",
            clauses.join(" AND ")
        ),
        &params,
    )?;

    struct RawRow {
        row_id: i64,
        created_at: String,
        at_ms: i64,
        display_name: Option<String>,
        previous_bio: Option<String>,
        bio: Option<String>,
    }
    let raw_rows: Vec<RawRow> = rows
        .into_iter()
        .filter_map(|row| {
            let row_id = row.first().and_then(Value::as_i64).unwrap_or(0);
            let created_at = row_string(&row, 1);
            let at_ms = parse_time_ms(&created_at)?;
            let display_name = row_string(&row, 2);
            let previous_bio = row_string(&row, 3);
            let bio = row_string(&row, 4);
            Some(RawRow {
                row_id,
                created_at,
                at_ms,
                display_name: (!display_name.is_empty()).then_some(display_name),
                previous_bio: (!previous_bio.is_empty()).then_some(previous_bio),
                bio: (!bio.is_empty()).then_some(bio),
            })
        })
        .collect();

    // Consecutive changes within the merge window collapse into one diff:
    // the base is the earliest previous_bio, the result the latest bio.
    // The window slides with the newest change, so a chain of edits each
    // less than `merge_hours` apart still merges into a single entry.
    let merge_ms = match query.merge_hours {
        Some(hours) if hours > 0 => hours * 60 * 60 * 1000,
        Some(_) => 0,
        None => DEFAULT_MERGE_HOURS * 60 * 60 * 1000,
    };
    let limit = if query.limit > 0 {
        query.limit.min(MAX_LIMIT)
    } else {
        DEFAULT_LIMIT
    };

    let mut merged: Vec<FeedBioHistoryRow> = Vec::new();
    let mut last_group_at_ms: Option<i64> = None;
    for raw in raw_rows {
        let joins_group = match last_group_at_ms {
            Some(last_at_ms) if merge_ms > 0 => raw.at_ms - last_at_ms <= merge_ms,
            _ => false,
        };
        if !joins_group {
            merged.push(FeedBioHistoryRow {
                row_id: raw.row_id,
                created_at: raw.created_at.clone(),
                first_created_at: raw.created_at,
                display_name: raw.display_name,
                previous_bio: raw.previous_bio,
                bio: raw.bio,
                merged_changes: 1,
            });
            last_group_at_ms = Some(raw.at_ms);
        } else if let Some(last) = merged.last_mut() {
            last.created_at = raw.created_at;
            last.row_id = raw.row_id;
            if raw.display_name.is_some() {
                last.display_name = raw.display_name;
            }
            last.bio = raw.bio;
            last.merged_changes += 1;
            last_group_at_ms = Some(raw.at_ms);
        }
    }
    merged.reverse();
    merged.truncate(limit as usize);
    Ok(FeedBioHistoryOutput { rows: merged })
}

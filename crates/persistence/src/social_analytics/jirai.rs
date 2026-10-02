//! VRCX-jirai parity queries over the desktop-recorded game log
//! (`gamelog_join_leave`), synced to the server: the two-person
//! relationship (shared-instance history between two friends) and the
//! relationship timeline rows (per friend per day social time).
//!
//! `OnPlayerLeft` rows carry `time` = how long the player stayed, so
//! join = leave - time and every interval is exact; occupancy peaks can be
//! swept over *all* players in a location, not just the friend roster.

use std::collections::HashMap;

use serde_json::Value;

use crate::common::{normalize_text, row_i64, row_string};
use crate::database::DatabaseService;
use crate::Error;

use vrcx_0_contracts::social_analytics::{
    RelationshipTimelineDayRow, RelationshipTimelineRowsInput, RelationshipTimelineRowsOutput,
    TwoPersonOverlapRow, TwoPersonRelationshipOutput, TwoPersonSelfPresenceRow,
};

fn owner_filter_param(db: &DatabaseService, owner_user_id: &str) -> Result<i64, Error> {
    crate::game_log::ensure_game_log_tables(db)?;
    let owner_row_id = crate::ownership::owner_id_for_filter(
        db,
        &vrcx_0_core::OwnerId::new(normalize_text(owner_user_id)),
    )?;
    Ok(owner_row_id.value())
}

pub fn two_person_relationship_query(
    db: &DatabaseService,
    owner_user_id: &str,
    user_id_a: &str,
    user_id_b: &str,
) -> Result<TwoPersonRelationshipOutput, Error> {
    let user_id_a = normalize_text(user_id_a);
    let user_id_b = normalize_text(user_id_b);
    if user_id_a.is_empty() || user_id_b.is_empty() || user_id_a == user_id_b {
        return Ok(TwoPersonRelationshipOutput::default());
    }
    let owner_row_id = owner_filter_param(db, owner_user_id)?;
    let mut params = HashMap::new();
    params.insert("@owner_id".into(), Value::from(owner_row_id));
    params.insert("@friendA".into(), Value::String(user_id_a.clone()));
    params.insert("@friendB".into(), Value::String(user_id_b.clone()));

    let rows = db.execute(
        "SELECT a.location,
                a.created_at AS friend_a_leave,
                a.time AS friend_a_time,
                b.created_at AS friend_b_leave,
                b.time AS friend_b_time
         FROM gamelog_join_leave a
         INNER JOIN gamelog_join_leave b
             ON a.location = b.location
         WHERE a.type = 'OnPlayerLeft'
           AND b.type = 'OnPlayerLeft'
           AND a.user_id = @friendA
           AND b.user_id = @friendB
           AND a.location NOT IN ('', 'traveling', 'private', 'private:private')
           AND b.location NOT IN ('', 'traveling', 'private', 'private:private')
           AND a.time > 0
           AND b.time > 0
           AND a.owner_id IN (0, @owner_id)
           AND b.owner_id IN (0, @owner_id)
           AND strftime('%Y-%m-%dT%H:%M:%SZ', a.created_at, '-' || (a.time * 1.0 / 1000) || ' seconds') < b.created_at
           AND strftime('%Y-%m-%dT%H:%M:%SZ', b.created_at, '-' || (b.time * 1.0 / 1000) || ' seconds') < a.created_at
         ORDER BY a.created_at DESC",
        &params,
    )?;
    let mut overlap = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut locations: Vec<String> = Vec::new();
    for row in rows {
        let entry = TwoPersonOverlapRow {
            location: row_string(&row, 0),
            friend_a_leave: row_string(&row, 1),
            friend_a_time: row_i64(&row, 2),
            friend_b_leave: row_string(&row, 3),
            friend_b_time: row_i64(&row, 4),
        };
        if entry.location.is_empty() {
            continue;
        }
        let key = format!(
            "{}|{}|{}|{}|{}",
            entry.location,
            entry.friend_a_leave,
            entry.friend_a_time,
            entry.friend_b_leave,
            entry.friend_b_time
        );
        if seen.insert(key) {
            if !locations.contains(&entry.location) {
                locations.push(entry.location.clone());
            }
            overlap.push(entry);
        }
    }

    if overlap.is_empty() {
        return Ok(TwoPersonRelationshipOutput::default());
    }

    // Self presence in the shared locations (own VRChat user id, not the
    // numeric owner row).
    let placeholders = locations
        .iter()
        .enumerate()
        .map(|(index, _)| format!("@loc{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut self_params = HashMap::new();
    self_params.insert(
        "@user_id".into(),
        Value::String(normalize_text(owner_user_id)),
    );
    for (index, location) in locations.iter().enumerate() {
        self_params.insert(format!("@loc{index}"), Value::String(location.clone()));
    }
    let self_rows = db.execute(
        &format!(
            "SELECT location, created_at, time FROM gamelog_join_leave
             WHERE user_id = @user_id
               AND location IN ({placeholders})
               AND type = 'OnPlayerLeft'
               AND time > 0
               AND owner_id IN (0, @owner_id)"
        ),
        &{
            let mut merged = HashMap::new();
            merged.insert("@owner_id".into(), Value::from(owner_row_id));
            for (key, value) in self_params {
                merged.insert(key, value);
            }
            merged
        },
    )?;
    let self_presence = self_rows
        .into_iter()
        .map(|row| TwoPersonSelfPresenceRow {
            location: row_string(&row, 0),
            self_leave: row_string(&row, 1),
            self_time: row_i64(&row, 2),
        })
        .filter(|row| !row.location.is_empty())
        .collect();

    // Peak occupancy per location: sweep every OnPlayerLeft record.
    let mut max_params = HashMap::new();
    max_params.insert("@owner_id".into(), Value::from(owner_row_id));
    for (index, location) in locations.iter().enumerate() {
        max_params.insert(format!("@loc{index}"), Value::String(location.clone()));
    }
    let sweep_rows = db.execute(
        &format!(
            "SELECT location, created_at, time FROM gamelog_join_leave
             WHERE location IN ({placeholders})
               AND type = 'OnPlayerLeft'
               AND time > 0
               AND owner_id IN (0, @owner_id)"
        ),
        &max_params,
    )?;
    let mut by_location: HashMap<String, Vec<(i64, i64)>> = HashMap::new();
    for row in sweep_rows {
        let location = row_string(&row, 0);
        let Some(leave_ms) = super::tracks::parse_time_ms(&row_string(&row, 1)) else {
            continue;
        };
        let duration = row_i64(&row, 2).max(0);
        by_location
            .entry(location)
            .or_default()
            .push((leave_ms - duration, leave_ms));
    }
    let mut max_player_counts = Vec::new();
    for (location, intervals) in by_location {
        let mut deltas: Vec<(i64, i32)> = Vec::with_capacity(intervals.len() * 2);
        for (join, leave) in intervals {
            deltas.push((join, 1));
            deltas.push((leave, -1));
        }
        // Exits before enters at the same instant (a leave-and-rejoin).
        deltas.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut running = 0i64;
        let mut peak = 0i64;
        for (_, delta) in deltas {
            running = (running + delta as i64).max(0);
            peak = peak.max(running);
        }
        max_player_counts.push((location, peak));
    }
    max_player_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    Ok(TwoPersonRelationshipOutput {
        rows: overlap,
        self_presence,
        max_player_counts,
    })
}

pub fn relationship_timeline_rows(
    db: &DatabaseService,
    input: RelationshipTimelineRowsInput,
) -> Result<RelationshipTimelineRowsOutput, Error> {
    let owner_user_id = normalize_text(&input.owner_user_id);
    if owner_user_id.is_empty() {
        return Ok(RelationshipTimelineRowsOutput::default());
    }
    let owner_row_id = owner_filter_param(db, &owner_user_id)?;
    let mut params = HashMap::new();
    params.insert("@owner_id".into(), Value::from(owner_row_id));
    params.insert("@currentUserId".into(), Value::String(owner_user_id));
    let rows = db.execute(
        "SELECT user_id,
                display_name,
                date(created_at) AS day,
                SUM(time) AS total_time,
                COUNT(DISTINCT location) AS join_count
         FROM gamelog_join_leave
         WHERE type = 'OnPlayerLeft'
           AND user_id != ''
           AND user_id != @currentUserId
           AND time > 0
           AND location NOT IN ('', 'traveling')
           AND owner_id IN (0, @owner_id)
         GROUP BY user_id, day
         ORDER BY day ASC",
        &params,
    )?;
    Ok(RelationshipTimelineRowsOutput {
        rows: rows
            .into_iter()
            .map(|row| RelationshipTimelineDayRow {
                user_id: row_string(&row, 0),
                display_name: row_string(&row, 1),
                day: row_string(&row, 2),
                total_time_ms: row_i64(&row, 3),
                join_count: row_i64(&row, 4),
            })
            .filter(|row| !row.user_id.is_empty() && !row.day.is_empty())
            .collect(),
    })
}

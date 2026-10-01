//! Fills empty `gamelog_join_leave.location` on timed leave rows with the
//! `gamelog_location` instance the stay started in.

use serde_json::Value;

use crate::activity::{activity_iso_from_ms, parse_activity_time_ms};
use crate::common::{row_i64, row_string, ParamsBuilder};
use crate::game_log::ensure_game_log_tables;
use crate::Error;

use super::super::DatabaseService;

const REPAIR_CHUNK_SIZE: usize = 5000;

pub(super) fn repair_empty_leave_locations(db: &DatabaseService) -> Result<(), Error> {
    ensure_game_log_tables(db)?;
    let empty_leaves = db.execute(
        "SELECT id, created_at, time, owner_id, user_id, display_name FROM gamelog_join_leave WHERE type = 'OnPlayerLeft' AND location = '' AND time > 0",
        &Default::default(),
    )?;
    for chunk in empty_leaves.chunks(REPAIR_CHUNK_SIZE) {
        db.write_transaction(|tx| {
            for row in chunk {
                let id = row.first().cloned().unwrap_or(Value::Null);
                let left_at = row_string(row, 1);
                let Some(left_ms) = parse_activity_time_ms(&left_at)
                    .filter(|&left_ms| activity_iso_from_ms(left_ms) == left_at)
                else {
                    continue;
                };
                let joined_ms = left_ms - row_i64(row, 2);
                let location_rows = tx.execute(
                    "SELECT started.location, started.created_at, started.time FROM (
                         SELECT created_at, location, time FROM gamelog_location
                         WHERE owner_id IN (0, @owner_id) AND created_at <= @joined_at
                         ORDER BY created_at DESC, id DESC LIMIT 1
                     ) started
                     WHERE started.location LIKE 'wrld_%'
                       AND NOT EXISTS (
                           SELECT 1 FROM gamelog_location later
                           WHERE later.owner_id IN (0, @owner_id)
                             AND later.created_at > started.created_at
                             AND later.created_at < @left_at
                       )
                       AND NOT EXISTS (
                           SELECT 1 FROM gamelog_join_leave joined
                           WHERE joined.type = 'OnPlayerJoined'
                             AND joined.created_at = @joined_at
                             AND ((@user_id <> '' AND joined.user_id = @user_id)
                                  OR (@user_id = '' AND joined.display_name = @display_name))
                             AND joined.location <> ''
                             AND joined.location <> started.location
                       )",
                    &ParamsBuilder::new()
                        .set("owner_id", row.get(3).cloned().unwrap_or(Value::Null))
                        .set("joined_at", activity_iso_from_ms(joined_ms))
                        .set("left_at", left_at)
                        .set("user_id", row_string(row, 4))
                        .set("display_name", row_string(row, 5))
                        .build(),
                )?;
                let Some(started) = location_rows.first() else {
                    continue;
                };
                let location_time = row_i64(started, 2);
                if location_time > 0
                    && parse_activity_time_ms(&row_string(started, 1))
                        .is_none_or(|started_ms| started_ms + location_time < joined_ms)
                {
                    continue;
                }
                tx.execute_non_query(
                    "UPDATE gamelog_join_leave SET location = @location WHERE id = @id",
                    &ParamsBuilder::new()
                        .set("location", row_string(started, 0))
                        .set("id", id)
                        .build(),
                )?;
            }
            Ok(())
        })?;
    }
    Ok(())
}

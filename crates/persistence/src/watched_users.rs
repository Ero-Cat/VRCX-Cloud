use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;

use crate::common::{normalize_text, now_iso, row_string, ParamsBuilder};
use crate::database::schema::ensure_user_store_tables;
use crate::database::DatabaseService;
use crate::realtime::normalize_user_table_prefix;
use crate::Error;

#[derive(Clone, Debug, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct WatchedUserOutput {
    pub user_id: String,
    pub display_name: String,
    pub added_at: String,
    pub last_polled_at: String,
    pub last_status: String,
    pub last_status_description: String,
    pub last_bio: String,
}

/// Last-known profile snapshot used by the poller to diff changes.
#[derive(Clone, Debug, Default)]
pub struct WatchedUserSnapshot {
    pub display_name: String,
    pub status: String,
    pub status_description: String,
    pub bio: String,
}

fn prefix_of(owner_user_id: &str) -> Result<String, Error> {
    normalize_user_table_prefix(owner_user_id.trim())
}

pub fn watched_user_add(
    db: &DatabaseService,
    owner_user_id: &str,
    target_user_id: &str,
    display_name: &str,
) -> Result<(), Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    let target_user_id = normalize_text(target_user_id);
    if !target_user_id.starts_with("usr_") {
        return Err(Error::Custom("Watched user id must be a usr_ id.".into()));
    }
    db.execute_non_query(
        &format!(
            "INSERT INTO {prefix}_watched_users (user_id, display_name, added_at)
             VALUES (@user_id, @display_name, @added_at)
             ON CONFLICT(user_id) DO UPDATE SET display_name = COALESCE(NULLIF(excluded.display_name, ''), {prefix}_watched_users.display_name)"
        ),
        &ParamsBuilder::new()
            .set("user_id", target_user_id)
            .set("display_name", normalize_text(display_name))
            .set("added_at", now_iso())
            .build(),
    )?;
    Ok(())
}

pub fn watched_user_remove(
    db: &DatabaseService,
    owner_user_id: &str,
    target_user_id: &str,
) -> Result<i64, Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    let target_user_id = normalize_text(target_user_id);
    if target_user_id.is_empty() {
        return Ok(0);
    }
    db.execute_non_query(
        &format!("DELETE FROM {prefix}_watched_users WHERE user_id = @user_id"),
        &ParamsBuilder::new().set("user_id", target_user_id).build(),
    )?;
    Ok(1)
}

pub fn watched_user_list(
    db: &DatabaseService,
    owner_user_id: &str,
) -> Result<Vec<WatchedUserOutput>, Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    let mut rows = db
        .execute(
            &format!(
                "SELECT user_id, display_name, added_at, last_polled_at, last_status, last_status_description, last_bio FROM {prefix}_watched_users ORDER BY added_at, user_id"
            ),
            &Default::default(),
        )?
        .into_iter()
        .map(|row| WatchedUserOutput {
            user_id: row_string(&row, 0),
            display_name: row_string(&row, 1),
            added_at: row_string(&row, 2),
            last_polled_at: row_string(&row, 3),
            last_status: row_string(&row, 4),
            last_status_description: row_string(&row, 5),
            last_bio: row_string(&row, 6),
        })
        .filter(|row| !row.user_id.is_empty())
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.added_at
            .cmp(&a.added_at)
            .then(a.display_name.cmp(&b.display_name))
    });
    Ok(rows)
}

pub fn watched_user_snapshot(
    db: &DatabaseService,
    owner_user_id: &str,
    target_user_id: &str,
) -> Result<Option<WatchedUserSnapshot>, Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    let rows = db.execute(
        &format!(
            "SELECT display_name, last_status, last_status_description, last_bio FROM {prefix}_watched_users WHERE user_id = @user_id"
        ),
        &ParamsBuilder::new()
            .set("user_id", normalize_text(target_user_id))
            .build(),
    )?;
    Ok(rows.first().map(|row| WatchedUserSnapshot {
        display_name: row_string(row, 0),
        status: row_string(row, 1),
        status_description: row_string(row, 2),
        bio: row_string(row, 3),
    }))
}

/// Oldest-polled watched user whose last poll predates `checked_before`.
pub fn watched_user_next_stale(
    db: &DatabaseService,
    owner_user_id: &str,
    checked_before: &str,
) -> Result<Option<String>, Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    let mut params = HashMap::new();
    params.insert(
        "@checked_before".into(),
        Value::String(checked_before.to_string()),
    );
    let rows = db.execute(
        &format!(
            "SELECT user_id FROM {prefix}_watched_users
             WHERE last_polled_at = '' OR last_polled_at < @checked_before
             ORDER BY last_polled_at, user_id LIMIT 1"
        ),
        &params,
    )?;
    Ok(rows
        .first()
        .map(|row| row_string(row, 0))
        .filter(|user_id| !user_id.is_empty()))
}

pub struct WatchedUserObservation<'a> {
    pub display_name: &'a str,
    pub status: &'a str,
    pub status_description: &'a str,
    pub bio: &'a str,
    pub polled_at: &'a str,
}

pub fn watched_user_record_observation(
    db: &DatabaseService,
    owner_user_id: &str,
    target_user_id: &str,
    observation: &WatchedUserObservation<'_>,
) -> Result<(), Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    db.execute_non_query(
        &format!(
            "UPDATE {prefix}_watched_users
             SET last_polled_at = @polled_at,
                 display_name = COALESCE(NULLIF(@display_name, ''), display_name),
                 last_status = @status,
                 last_status_description = @status_description,
                 last_bio = @bio
             WHERE user_id = @user_id"
        ),
        &ParamsBuilder::new()
            .set("user_id", normalize_text(target_user_id))
            .set("display_name", normalize_text(observation.display_name))
            .set("status", normalize_text(observation.status))
            .set(
                "status_description",
                normalize_text(observation.status_description),
            )
            .set("bio", observation.bio.to_string())
            .set("polled_at", observation.polled_at.to_string())
            .build(),
    )?;
    Ok(())
}

/// Advance the poll timestamp without touching the last-known values
/// (used when the profile was unavailable on this poll).
pub fn watched_user_mark_polled(
    db: &DatabaseService,
    owner_user_id: &str,
    target_user_id: &str,
    polled_at: &str,
) -> Result<(), Error> {
    let prefix = prefix_of(owner_user_id)?;
    ensure_user_store_tables(db, &prefix)?;
    db.execute_non_query(
        &format!(
            "UPDATE {prefix}_watched_users SET last_polled_at = @polled_at WHERE user_id = @user_id"
        ),
        &ParamsBuilder::new()
            .set("user_id", normalize_text(target_user_id))
            .set("polled_at", polled_at.to_string())
            .build(),
    )?;
    Ok(())
}

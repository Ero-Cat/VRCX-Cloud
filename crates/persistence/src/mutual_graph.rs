use serde::{Deserialize, Serialize};

use crate::common::{normalize_text, now_iso, row_i64, row_string, row_value, ParamsBuilder};
use crate::database::schema::ensure_user_store_tables;
use crate::database::{DatabaseService, DatabaseWriteTransaction};
use crate::realtime::normalize_user_table_prefix;
use crate::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphSnapshotEntryInput {
    pub friend_id: String,
    #[serde(default)]
    pub mutual_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphMetaInput {
    pub friend_id: String,
    #[serde(default)]
    pub last_fetched_at: String,
    #[serde(default)]
    pub opted_out: bool,
    #[serde(default)]
    pub total_count: Option<usize>,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphLinkOutput {
    pub friend_id: String,
    pub mutual_id: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphManualLinkOutput {
    pub friend_id: String,
    pub mutual_id: String,
    pub note: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphExternalUserOutput {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: String,
    pub added_at: String,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphMetaOutput {
    pub friend_id: String,
    pub last_fetched_at: String,
    pub opted_out: bool,
    pub total_count: Option<u32>,
}

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MutualGraphSnapshotOutput {
    pub friend_ids: Vec<String>,
    pub links: Vec<MutualGraphLinkOutput>,
    pub meta: Vec<MutualGraphMetaOutput>,
    /// User-drawn links, kept separate from the API-derived tables so the
    /// full-replace snapshot refresh never touches them.
    pub manual_links: Vec<MutualGraphManualLinkOutput>,
    /// Non-friend nodes the user pinned into the graph.
    pub external_users: Vec<MutualGraphExternalUserOutput>,
}

pub fn mutual_graph_snapshot_get(
    db: &DatabaseService,
    user_id: String,
) -> Result<MutualGraphSnapshotOutput, Error> {
    let user_id = normalize_text(user_id);
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;

    let friend_ids = db
        .execute(
            &format!("SELECT friend_id FROM {user_prefix}_mutual_graph_friends"),
            &Default::default(),
        )?
        .into_iter()
        .map(|row| row_string(&row, 0))
        .filter(|friend_id| !friend_id.is_empty())
        .collect();
    let links = db
        .execute(
            &format!("SELECT friend_id, mutual_id FROM {user_prefix}_mutual_graph_links"),
            &Default::default(),
        )?
        .into_iter()
        .filter_map(|row| {
            let friend_id = row_string(&row, 0);
            let mutual_id = row_string(&row, 1);
            if friend_id.is_empty() || mutual_id.is_empty() {
                None
            } else {
                Some(MutualGraphLinkOutput {
                    friend_id,
                    mutual_id,
                })
            }
        })
        .collect();
    let meta = db
        .execute(
            &format!(
                "SELECT friend_id, last_fetched_at, opted_out, total_count FROM {user_prefix}_mutual_graph_meta"
            ),
            &Default::default(),
        )?
        .into_iter()
        .filter_map(|row| {
            let friend_id = row_string(&row, 0);
            if friend_id.is_empty() {
                None
            } else {
                Some(MutualGraphMetaOutput {
                    friend_id,
                    last_fetched_at: row_string(&row, 1),
                    opted_out: row_i64(&row, 2) == 1,
                    total_count: row_value(&row, 3)
                        .as_i64()
                        .and_then(|count| u32::try_from(count).ok()),
                })
            }
        })
        .collect();
    let manual_links = db
        .execute(
            &format!(
                "SELECT friend_id, mutual_id, note, created_at FROM {user_prefix}_mutual_graph_manual_links"
            ),
            &Default::default(),
        )?
        .into_iter()
        .filter_map(|row| {
            let friend_id = row_string(&row, 0);
            let mutual_id = row_string(&row, 1);
            if friend_id.is_empty() || mutual_id.is_empty() {
                return None;
            }
            Some(MutualGraphManualLinkOutput {
                friend_id,
                mutual_id,
                note: row_string(&row, 2),
                created_at: row_string(&row, 3),
            })
        })
        .collect();
    let external_users = db
        .execute(
            &format!(
                "SELECT user_id, display_name, avatar_url, added_at FROM {user_prefix}_mutual_graph_external_users"
            ),
            &Default::default(),
        )?
        .into_iter()
        .filter_map(|row| {
            let user_id = row_string(&row, 0);
            (!user_id.is_empty()).then_some(MutualGraphExternalUserOutput {
                user_id,
                display_name: row_string(&row, 1),
                avatar_url: row_string(&row, 2),
                added_at: row_string(&row, 3),
            })
        })
        .collect();

    Ok(MutualGraphSnapshotOutput {
        friend_ids,
        links,
        meta,
        manual_links,
        external_users,
    })
}

fn insert_mutual_graph_friend(
    tx: &mut DatabaseWriteTransaction<'_>,
    user_prefix: &str,
    friend_id: &str,
) -> Result<(), crate::Error> {
    tx.execute_non_query(
        &format!("INSERT OR REPLACE INTO {user_prefix}_mutual_graph_friends (friend_id) VALUES (@friend_id)"),
        &ParamsBuilder::new().set("friend_id", friend_id.to_string()).build(),
    )?;
    Ok(())
}

fn insert_mutual_graph_link(
    tx: &mut DatabaseWriteTransaction<'_>,
    user_prefix: &str,
    friend_id: &str,
    mutual_id: &str,
) -> Result<(), crate::Error> {
    tx.execute_non_query(
        &format!("INSERT OR REPLACE INTO {user_prefix}_mutual_graph_links (friend_id, mutual_id) VALUES (@friend_id, @mutual_id)"),
        &ParamsBuilder::new()
            .set("friend_id", friend_id.to_string())
            .set("mutual_id", mutual_id.to_string())
            .build(),
    )?;
    Ok(())
}

fn upsert_mutual_graph_meta_entries(
    tx: &mut DatabaseWriteTransaction<'_>,
    user_prefix: &str,
    entries: &[MutualGraphMetaInput],
) -> Result<(), Error> {
    let now = now_iso();
    for entry in entries {
        let friend_id = normalize_text(&entry.friend_id);
        if friend_id.is_empty() {
            continue;
        }
        tx.execute_non_query(
            &format!("INSERT INTO {user_prefix}_mutual_graph_meta (friend_id, last_fetched_at, opted_out, total_count) VALUES (@friend_id, @last_fetched_at, @opted_out, @total_count) ON CONFLICT(friend_id) DO UPDATE SET last_fetched_at = excluded.last_fetched_at, opted_out = excluded.opted_out, total_count = COALESCE(excluded.total_count, {user_prefix}_mutual_graph_meta.total_count)"),
            &ParamsBuilder::new()
                .set("friend_id", friend_id)
                .set(
                    "last_fetched_at",
                    if entry.last_fetched_at.trim().is_empty() {
                        now.clone()
                    } else {
                        entry.last_fetched_at.clone()
                    },
                )
                .set("opted_out", if entry.opted_out { 1 } else { 0 })
                .set(
                    "total_count",
                    match entry.total_count {
                        Some(total_count) => {
                            serde_json::Value::from(i64::try_from(total_count).unwrap_or(i64::MAX))
                        }
                        None => serde_json::Value::Null,
                    },
                )
                .build(),
        )?;
    }
    Ok(())
}

fn replace_mutual_graph_snapshot_entries(
    tx: &mut DatabaseWriteTransaction<'_>,
    user_prefix: &str,
    entries: &[MutualGraphSnapshotEntryInput],
) -> Result<(), Error> {
    tx.execute_non_query(
        &format!("DELETE FROM {user_prefix}_mutual_graph_links WHERE friend_id NOT IN (SELECT friend_id FROM {user_prefix}_mutual_graph_meta WHERE opted_out = 1)"),
        &Default::default(),
    )?;
    tx.execute_non_query(
        &format!("DELETE FROM {user_prefix}_mutual_graph_friends WHERE friend_id NOT IN (SELECT friend_id FROM {user_prefix}_mutual_graph_meta WHERE opted_out = 1)"),
        &Default::default(),
    )?;
    for entry in entries {
        let friend_id = normalize_text(&entry.friend_id);
        if friend_id.is_empty() {
            continue;
        }
        tx.execute_non_query(
            &format!("DELETE FROM {user_prefix}_mutual_graph_links WHERE friend_id = @friend_id"),
            &ParamsBuilder::new()
                .set("friend_id", friend_id.clone())
                .build(),
        )?;
        insert_mutual_graph_friend(tx, user_prefix, &friend_id)?;
        for mutual_id in &entry.mutual_ids {
            let mutual_id = normalize_text(mutual_id);
            if !mutual_id.is_empty() {
                insert_mutual_graph_link(tx, user_prefix, &friend_id, &mutual_id)?;
            }
        }
    }
    Ok(())
}

pub fn mutual_graph_snapshot_commit(
    db: &DatabaseService,
    user_id: String,
    entries: Vec<MutualGraphSnapshotEntryInput>,
    meta_entries: Vec<MutualGraphMetaInput>,
) -> Result<(), Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    db.write_transaction(|tx| {
        tx.execute_non_query(
            &format!("DELETE FROM {user_prefix}_mutual_graph_meta"),
            &Default::default(),
        )?;
        upsert_mutual_graph_meta_entries(tx, &user_prefix, &meta_entries)?;
        replace_mutual_graph_snapshot_entries(tx, &user_prefix, &entries)?;
        Ok(())
    })?;
    Ok(())
}

pub fn mutual_graph_friend_refresh_commit(
    db: &DatabaseService,
    user_id: String,
    friend_id: String,
    mutual_ids: Option<Vec<String>>,
    total_count: Option<usize>,
    opted_out: bool,
) -> Result<(), Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    let friend_id = normalize_text(friend_id);
    if friend_id.is_empty() {
        return Ok(());
    }
    db.write_transaction(|tx| {
        if let Some(mutual_ids) = mutual_ids {
            insert_mutual_graph_friend(tx, &user_prefix, &friend_id)?;
            tx.execute_non_query(
                &format!(
                    "DELETE FROM {user_prefix}_mutual_graph_links WHERE friend_id = @friend_id"
                ),
                &ParamsBuilder::new()
                    .set("friend_id", friend_id.clone())
                    .build(),
            )?;
            for mutual_id in mutual_ids {
                let mutual_id = normalize_text(mutual_id);
                if !mutual_id.is_empty() {
                    insert_mutual_graph_link(tx, &user_prefix, &friend_id, &mutual_id)?;
                }
            }
        }
        upsert_mutual_graph_meta_entries(
            tx,
            &user_prefix,
            &[MutualGraphMetaInput {
                friend_id,
                last_fetched_at: String::new(),
                opted_out,
                total_count,
            }],
        )?;
        Ok(())
    })?;
    Ok(())
}

pub fn mutual_graph_manual_link_add(
    db: &DatabaseService,
    user_id: String,
    friend_id: String,
    mutual_id: String,
    note: String,
) -> Result<(), Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    let friend_id = normalize_text(&friend_id);
    let mutual_id = normalize_text(&mutual_id);
    if friend_id.is_empty() || mutual_id.is_empty() || friend_id == mutual_id {
        return Ok(());
    }
    // Store the pair canonically so (a, b) and (b, a) are one link.
    let (left, right) = if friend_id < mutual_id {
        (friend_id, mutual_id)
    } else {
        (mutual_id, friend_id)
    };
    db.execute_non_query(
        &format!(
            "INSERT OR IGNORE INTO {user_prefix}_mutual_graph_manual_links (friend_id, mutual_id, note, created_at)
             VALUES (@friend_id, @mutual_id, @note, @created_at)"
        ),
        &ParamsBuilder::new()
            .set("friend_id", left)
            .set("mutual_id", right)
            .set("note", normalize_text(&note))
            .set("created_at", now_iso())
            .build(),
    )?;
    Ok(())
}

pub fn mutual_graph_manual_link_remove(
    db: &DatabaseService,
    user_id: String,
    friend_id: String,
    mutual_id: String,
) -> Result<i64, Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    let friend_id = normalize_text(&friend_id);
    let mutual_id = normalize_text(&mutual_id);
    if friend_id.is_empty() || mutual_id.is_empty() {
        return Ok(0);
    }
    let (left, right) = if friend_id < mutual_id {
        (friend_id, mutual_id)
    } else {
        (mutual_id, friend_id)
    };
    db.execute_non_query(
        &format!(
            "DELETE FROM {user_prefix}_mutual_graph_manual_links WHERE friend_id = @friend_id AND mutual_id = @mutual_id"
        ),
        &ParamsBuilder::new()
            .set("friend_id", left)
            .set("mutual_id", right)
            .build(),
    )?;
    Ok(1)
}

pub fn mutual_graph_external_user_add(
    db: &DatabaseService,
    user_id: String,
    target_user_id: String,
    display_name: String,
    avatar_url: String,
) -> Result<(), Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    let target_user_id = normalize_text(&target_user_id);
    if target_user_id.is_empty() {
        return Ok(());
    }
    db.execute_non_query(
        &format!(
            "INSERT INTO {user_prefix}_mutual_graph_external_users (user_id, display_name, avatar_url, added_at)
             VALUES (@user_id, @display_name, @avatar_url, @added_at)
             ON CONFLICT(user_id) DO UPDATE SET display_name = excluded.display_name, avatar_url = excluded.avatar_url"
        ),
        &ParamsBuilder::new()
            .set("user_id", target_user_id)
            .set("display_name", normalize_text(&display_name))
            .set("avatar_url", normalize_text(&avatar_url))
            .set("added_at", now_iso())
            .build(),
    )?;
    Ok(())
}

pub fn mutual_graph_external_user_remove(
    db: &DatabaseService,
    user_id: String,
    target_user_id: String,
) -> Result<i64, Error> {
    let user_prefix = normalize_user_table_prefix(&user_id)?;
    ensure_user_store_tables(db, &user_prefix)?;
    let target_user_id = normalize_text(&target_user_id);
    if target_user_id.is_empty() {
        return Ok(0);
    }
    db.execute_non_query(
        &format!("DELETE FROM {user_prefix}_mutual_graph_external_users WHERE user_id = @user_id"),
        &ParamsBuilder::new().set("user_id", target_user_id).build(),
    )?;
    Ok(1)
}

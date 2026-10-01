use crate::activity::activity_iso_from_ms;
use crate::common::{row_string, ParamsBuilder};
use crate::database::DatabaseService;
use crate::game_log::ensure_game_log_tables;
use crate::ownership::{owner_id_for_filter, OwnerId};
use crate::Error;
use std::collections::BTreeSet;

pub fn first_source_created_at(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
) -> Result<String, Error> {
    ensure_game_log_tables(db)?;
    Ok(db
        .execute(
            "SELECT MIN(created_at) FROM gamelog_location WHERE owner_id IN (0, @owner_id)",
            &ParamsBuilder::new()
                .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
                .build(),
        )?
        .first()
        .map(|row| row_string(row, 0))
        .unwrap_or_default())
}

pub fn world_ids_before(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    before_ms: i64,
) -> Result<BTreeSet<String>, Error> {
    ensure_game_log_tables(db)?;
    Ok(db
        .execute(
            "SELECT DISTINCT world_id
             FROM gamelog_location
             WHERE owner_id IN (0, @owner_id)
               AND created_at < @before_iso
               AND world_id LIKE 'wrld_%'",
            &ParamsBuilder::new()
                .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
                .set("before_iso", activity_iso_from_ms(before_ms))
                .build(),
        )?
        .into_iter()
        .map(|row| row_string(&row, 0))
        .filter(|world_id| !world_id.is_empty())
        .collect())
}

pub fn encountered_user_ids(
    db: &DatabaseService,
    owner_user_id: &OwnerId,
    from_ms: Option<i64>,
    to_ms: Option<i64>,
) -> Result<BTreeSet<String>, Error> {
    ensure_game_log_tables(db)?;
    let mut sql = String::from(
        "SELECT DISTINCT user_id
         FROM gamelog_join_leave
         WHERE owner_id IN (0, @owner_id)
           AND trim(user_id) <> ''
           AND user_id <> @owner_user_id",
    );
    let mut params = ParamsBuilder::new()
        .set("owner_id", owner_id_for_filter(db, owner_user_id)?)
        .set("owner_user_id", owner_user_id.as_str());
    if let Some(from_ms) = from_ms {
        sql.push_str(" AND created_at >= @from_iso");
        params = params.set("from_iso", activity_iso_from_ms(from_ms));
    }
    if let Some(to_ms) = to_ms {
        sql.push_str(" AND created_at < @to_iso");
        params = params.set("to_iso", activity_iso_from_ms(to_ms));
    }
    Ok(db
        .execute(&sql, &params.build())?
        .into_iter()
        .map(|row| row_string(&row, 0))
        .filter(|user_id| !user_id.is_empty())
        .collect())
}

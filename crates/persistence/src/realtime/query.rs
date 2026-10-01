use crate::common::ParamsBuilder;
use crate::database::DatabaseService;
use crate::game_log::ensure_game_log_tables;
use crate::Error;

pub fn lookup_game_log_world_name(db: &DatabaseService, world_id: &str) -> Result<String, Error> {
    let world_id = world_id.trim();
    if world_id.is_empty() {
        return Ok(String::new());
    }
    ensure_game_log_tables(db)?;
    let rows = db.execute(
        "SELECT world_name FROM gamelog_location WHERE world_id = @world_id ORDER BY id DESC LIMIT 1",
        &ParamsBuilder::new().set("world_id", world_id).build(),
    )?;
    Ok(rows
        .first()
        .and_then(|row| row.first())
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string())
}

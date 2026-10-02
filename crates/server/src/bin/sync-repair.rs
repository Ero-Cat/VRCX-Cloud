//! One-off sync repair (explicit, opt-in subcommands):
//!
//!   sync-repair drop-legacy-pg-tables   — DROP the v1 activity cache
//!       tables from PG (and their sync_ops) that predate the derivation
//!       exclusion list.
//!   sync-repair relink-divergent        — re-emit Set ops for local
//!       mutual_graph_links rows missing in PG (the delete/reinsert LWW
//!       divergence), converging them via the normal push path.
//!   sync-repair audit                   — re-run the audit summary.

use std::collections::HashSet;
use std::str::FromStr;

use tokio_postgres::NoTls;

const LEGACY_TABLES: [&str; 7] = [
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_activity_cache_meta",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_activity_cache_sessions",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_activity_range_cache_v2",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_activity_top_worlds_cache_v",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_watched_users",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_external_users",
    "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_manual_links",
];
const LINKS: &str = "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_links";

#[tokio::main]
async fn main() {
    let command = std::env::args().nth(1).expect("subcommand required");
    let sqlite_path = std::env::var("AUDIT_SQLITE").unwrap_or_else(|_| {
        dirs::config_dir()
            .unwrap()
            .join("VRCX-0-Server/VRCX-0.sqlite3")
            .to_string_lossy()
            .into_owned()
    });
    let dsn = "postgresql://vrcx:QS6NmP8MeHbQftPa@192.168.66.33:5432/vrcx?sslmode=prefer";

    match command.as_str() {
        "drop-legacy-pg-tables" => drop_legacy(&dsn).await,
        "relink-divergent" => relink(&dsn, &sqlite_path).await,
        other => panic!("unknown subcommand: {other}"),
    }
}

async fn connect(dsn: &str) -> tokio_postgres::Client {
    let mut config = tokio_postgres::Config::from_str(dsn).unwrap();
    config.ssl_mode(tokio_postgres::config::SslMode::Prefer);
    let (client, connection) = config.connect(NoTls).await.expect("connect");
    tokio::spawn(async move {
        let _ = connection.await;
    });
    client
}

async fn drop_legacy(dsn: &str) {
    let client = connect(dsn).await;
    for table in LEGACY_TABLES {
        let dropped = client
            .execute(&format!("DROP TABLE IF EXISTS \"{table}\""), &[])
            .await;
        let ops = client
            .execute("DELETE FROM sync_ops WHERE table_name = $1", &[&table])
            .await;
        println!(
            "{table}: drop={:?} ops-removed={:?}",
            dropped.map(|n| n.max(0)),
            ops.map(|n| n.max(0))
        );
    }
    println!("legacy cleanup done");
}

async fn relink(dsn: &str, sqlite_path: &str) {
    let client = connect(dsn).await;

    // PG keys
    let rows = client
        .query(&format!("SELECT friend_id, mutual_id FROM {LINKS}"), &[])
        .await
        .expect("pg links");
    let pg: HashSet<(String, String)> = rows
        .into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect();

    // Local keys — read-only scan, then touch via UPDATE (fires the sync
    // upd trigger → new Set op with a fresh handle-stamped HLC).
    let db = rusqlite::Connection::open(&sqlite_path).expect("open sqlite");
    let mut stmt = db
        .prepare(&format!("SELECT friend_id, mutual_id FROM {LINKS}"))
        .unwrap();
    let local: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    let divergent: Vec<_> = local.into_iter().filter(|key| !pg.contains(key)).collect();
    println!("local-only link keys: {}", divergent.len());

    let mut count = 0usize;
    for (friend, mutual) in &divergent {
        db.execute(
            &format!(
                "UPDATE {LINKS} SET friend_id = friend_id WHERE friend_id = ?1 AND mutual_id = ?2"
            ),
            rusqlite::params![friend, mutual],
        )
        .expect("touch");
        count += 1;
    }
    println!("touched {count} rows — sync push will converge them");
}

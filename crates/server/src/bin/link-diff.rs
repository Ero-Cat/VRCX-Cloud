//! One-off diagnostic: compare mutual_graph_links key sets between the
//! local SQLite and the remote PG to identify the 175-row divergence.

use std::collections::HashSet;
use std::str::FromStr;

use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let dsn = "postgresql://vrcx:QS6NmP8MeHbQftPa@192.168.66.33:5432/vrcx?sslmode=prefer";
    let sqlite_path = "/Users/erocat/Library/Application Support/VRCX-0-Server/VRCX-0.sqlite3";
    let sqlite_path = std::env::var("AUDIT_SQLITE").unwrap_or_else(|_| sqlite_path.to_string());

    let mut config = tokio_postgres::Config::from_str(dsn).unwrap();
    config.ssl_mode(tokio_postgres::config::SslMode::Prefer);
    let (client, connection) = config.connect(NoTls).await.expect("connect");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let rows = client
        .query(
            "SELECT friend_id, mutual_id FROM usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_links",
            &[],
        )
        .await
        .expect("query pg");
    let pg: HashSet<(String, String)> = rows
        .into_iter()
        .map(|row| {
            let f: String = row.get(0);
            let m: String = row.get(1);
            (f, m)
        })
        .collect();
    println!("pg links: {}", pg.len());

    let db = rusqlite::Connection::open_with_flags(
        &sqlite_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .expect("open sqlite");
    let mut stmt = db
        .prepare("SELECT friend_id, mutual_id FROM usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_links")
        .expect("prepare");
    let local: HashSet<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .expect("query")
        .filter_map(Result::ok)
        .collect();
    println!("local links: {}", local.len());

    let only_local: Vec<_> = local.difference(&pg).collect();
    let only_pg: Vec<_> = pg.difference(&local).collect();
    println!("only-local: {}", only_local.len());
    println!("only-pg:    {}", only_pg.len());

    println!("\nsample only-local (friend_id, mutual_id):");
    for key in only_local.iter().take(8) {
        println!("  {:?}", key);
    }
    println!("\nsample only-pg:");
    for key in only_pg.iter().take(8) {
        println!("  {:?}", key);
    }

    // Check sync_device/sync_hlc of a few only-local rows in... they're
    // local-only, so check _sync_row_version hlc to see if they were pushed
    let mut stmt = db
        .prepare(
            "SELECT v.hlc FROM _sync_row_version v
             WHERE v.table_name LIKE '%mutual_graph_links'
             ORDER BY v.hlc DESC LIMIT 3",
        )
        .expect("prepare version");
    let hlcs: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("q")
        .filter_map(Result::ok)
        .collect();
    println!("\nnewest local row hlcs: {:?}", hlcs);
}

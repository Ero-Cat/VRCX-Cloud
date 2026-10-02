//! Read-only sync audit: verifies the remote PostgreSQL and the local
//! (server) SQLite converged without duplicates or dirty data.
//!
//! Usage:
//!   cargo run --release -p vrcx-0-server --bin sync-audit \
//!       [sqlite_path] [--dsn postgresql://...]
//!
//! Defaults: SQLite path = the VRCX-0-Server data dir; DSN from
//! VRCX_CLOUD_SYNC_* composition. Never writes to either database.

use std::collections::HashMap;
use std::str::FromStr;

use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let mut sqlite_path = None;
    let mut dsn = std::env::var("VRCX_PG_TEST_DSN").unwrap_or_else(|_| {
        format!(
            "postgresql://vrcx:{}@192.168.66.33:5432/vrcx",
            "QS6NmP8MeHbQftPa"
        )
    });
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dsn" => {
                dsn = args.next().expect("--dsn requires a value");
            }
            path => sqlite_path = Some(path.to_string()),
        }
    }
    let sqlite_path = sqlite_path.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("VRCX-0-Server/VRCX-0.sqlite3")
            .to_string_lossy()
            .into_owned()
    });

    println!("=== sync-audit (read-only) ===");
    println!("sqlite: {sqlite_path}");
    println!("dsn:    {dsn}\n");

    let mut findings: Vec<String> = Vec::new();

    let pg = connect_pg(&dsn).await;
    audit_protocol(&pg, &mut findings).await;
    audit_pg_tables(&pg, &mut findings).await;
    let pg_counts = pg_table_counts(&pg).await;
    let sqlite_counts = sqlite_table_counts(&sqlite_path, &mut findings);
    compare_counts(&pg_counts, &sqlite_counts, &mut findings);

    println!("\n=== SUMMARY ===");
    if findings.is_empty() {
        println!("✅ no anomalies found");
    } else {
        println!("⚠️  {} finding(s):", findings.len());
        for (index, finding) in findings.iter().enumerate() {
            println!("  {}. {}", index + 1, finding);
        }
        println!("\n(audit only — nothing was modified)");
    }
}

async fn connect_pg(dsn: &str) -> tokio_postgres::Client {
    let mut config = tokio_postgres::Config::from_str(dsn).expect("parse DSN");
    config.ssl_mode(tokio_postgres::config::SslMode::Prefer);
    let (client, connection) = config.connect(NoTls).await.expect("connect PG");
    tokio::spawn(async move {
        let _ = connection.await;
    });
    client
}

async fn scalar_i64(pg: &tokio_postgres::Client, sql: &str) -> i64 {
    let row = pg.query_one(sql, &[]).await.expect("query");
    row.try_get::<_, i64>(0).unwrap_or(0)
}

async fn audit_protocol(pg: &tokio_postgres::Client, findings: &mut Vec<String>) {
    println!("--- protocol tables ---");
    let ops = scalar_i64(pg, "SELECT count(*) FROM sync_ops").await;
    let devices = scalar_i64(pg, "SELECT count(*) FROM _sync_devices").await;
    let min_seq = scalar_i64(pg, "SELECT coalesce(min(server_seq),0) FROM sync_ops").await;
    let max_seq = scalar_i64(pg, "SELECT coalesce(max(server_seq),0) FROM sync_ops").await;
    println!("sync_ops: {ops} (seq {min_seq}..{max_seq})  devices: {devices}");

    // op_id duplicates are structurally impossible (UNIQUE) but verify count
    let distinct_ops = scalar_i64(pg, "SELECT count(distinct op_id) FROM sync_ops").await;
    if distinct_ops != ops {
        findings.push(format!(
            "sync_ops: {ops} rows but {distinct_ops} distinct op_id (duplicates present)"
        ));
    }

    // malformed entity_key / empty hlc / unknown device
    let bad_keys = scalar_i64(
        pg,
        "SELECT count(*) FROM sync_ops WHERE entity_key IS NULL OR entity_key = '' OR hlc = ''",
    )
    .await;
    if bad_keys > 0 {
        findings.push(format!(
            "sync_ops: {bad_keys} rows with empty entity_key/hlc"
        ));
    }
    let orphan_devices = scalar_i64(
        pg,
        "SELECT count(*) FROM sync_ops s WHERE NOT EXISTS (SELECT 1 FROM _sync_devices d WHERE d.device_id = s.device)",
    )
    .await;
    if orphan_devices > 0 {
        findings.push(format!(
            "sync_ops: {orphan_devices} rows from unknown devices"
        ));
    }

    // ops referencing tables with no materialized counterpart
    let orphan_tables = pg
        .query(
            "SELECT DISTINCT table_name, count(*) FROM sync_ops s
             WHERE NOT EXISTS (
                 SELECT 1 FROM information_schema.tables t
                 WHERE t.table_schema='public' AND t.table_name = s.table_name
             ) GROUP BY table_name",
            &[],
        )
        .await
        .expect("orphan tables");
    for row in &orphan_tables {
        let table: String = row.get(0);
        let count: i64 = row.get(1);
        findings.push(format!(
            "sync_ops: {count} ops reference table `{table}` which has no materialized table"
        ));
    }

    // device sanity
    let stale_devices = scalar_i64(
        pg,
        "SELECT count(*) FROM _sync_devices WHERE last_seen_at < now() - interval '7 days'",
    )
    .await;
    if stale_devices > 0 {
        println!("note: {stale_devices} device(s) unseen for >7 days (past test devices)");
    }
}

async fn pg_table_counts(pg: &tokio_postgres::Client) -> HashMap<String, i64> {
    let mut counts = HashMap::new();
    let rows = pg
        .query(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema='public' AND table_type='BASE TABLE'
               AND table_name NOT IN ('_sync_meta','sync_ops','_sync_devices')
             ORDER BY table_name",
            &[],
        )
        .await
        .expect("list tables");
    for row in rows {
        let table: String = row.get(0);
        let count = scalar_i64(pg, &format!("SELECT count(*) FROM \"{table}\"")).await;
        counts.insert(table, count);
    }
    counts
}

async fn audit_pg_tables(pg: &tokio_postgres::Client, findings: &mut Vec<String>) {
    println!("--- PG materialized tables ---");
    let rows = pg
        .query(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema='public' AND table_type='BASE TABLE'
               AND table_name NOT IN ('_sync_meta','sync_ops','_sync_devices')
             ORDER BY table_name",
            &[],
        )
        .await
        .expect("list tables");
    for row in rows {
        let table: String = row.get(0);
        let count = scalar_i64(pg, &format!("SELECT count(*) FROM \"{table}\"")).await;
        // Detect duplicate natural keys: rows sharing all columns except the
        // protocol watermark would have collided on UNIQUE during upsert, so
        // duplicates manifest as repeated business keys WITHOUT the unique
        // index. Approximate with full-row dupes:
        let col_rows = pg
            .query(
                "SELECT column_name FROM information_schema.columns
                 WHERE table_schema='public' AND table_name=$1
                   AND column_name NOT IN ('sync_hlc','sync_device')
                 ORDER BY ordinal_position",
                &[&table],
            )
            .await
            .expect("columns");
        let cols: Vec<String> = col_rows
            .iter()
            .map(|r| r.try_get::<_, String>(0).unwrap_or_default())
            .collect();
        if cols.is_empty() {
            continue;
        }
        let col_list = cols
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let dup_sql = format!(
            "SELECT count(*) FROM (SELECT {col_list}, count(*) c FROM \"{table}\"
             GROUP BY {col_list} HAVING count(*) > 1) d"
        );
        let dupes = scalar_i64(pg, &dup_sql).await;
        // rows with empty hlc => inserted outside the sync protocol
        let raw = scalar_i64(
            pg,
            &format!("SELECT count(*) FROM \"{table}\" WHERE sync_hlc = ''"),
        )
        .await;
        println!("  {table}: {count} rows, dup-keys={dupes}, protocol-foreign={raw}");
        if dupes > 0 {
            findings.push(format!(
                "PG `{table}`: {dupes} duplicated business-key group(s)",
            ));
        }
        if raw > 0 {
            findings.push(format!(
                "PG `{table}`: {raw} rows with empty sync_hlc (written outside the sync protocol)"
            ));
        }
    }
}

fn sqlite_table_counts(path: &str, findings: &mut Vec<String>) -> HashMap<String, i64> {
    println!("\n--- SQLite (server side) ---");
    let mut counts = HashMap::new();
    let Ok(db) =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    else {
        findings.push(format!("cannot open SQLite at {path} (read-only)"));
        return counts;
    };
    let mut prepared = db
        .prepare(
            "SELECT name FROM sqlite_master WHERE type='table'
             AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '\\_%' ESCAPE '\\'
             ORDER BY name",
        )
        .expect("list tables");
    let tables: Vec<String> = prepared
        .query_map([], |row| row.get(0))
        .expect("query")
        .filter_map(Result::ok)
        .collect();
    for table in tables {
        let quoted = table.replace('\"', "\"\"");
        let count: i64 = db
            .query_row(&format!("SELECT count(*) FROM \"{quoted}\""), [], |row| {
                row.get(0)
            })
            .unwrap_or(-1);
        counts.insert(table, count);
    }
    println!(
        "  {} tables, total rows: {}",
        counts.len(),
        counts.values().filter(|c| **c > 0).count()
    );
    counts
}

fn compare_counts(
    pg: &HashMap<String, i64>,
    sqlite: &HashMap<String, i64>,
    findings: &mut Vec<String>,
) {
    println!("\n--- convergence: PG vs SQLite ---");
    let mut checked = 0i64;
    let mut mismatches = 0i64;
    // Skip derived/local-only tables on the SQLite side.
    let sqlite_only: Vec<&str> = vec![
        "_sync_outbox",
        "_sync_row_version",
        "cookies",
        "favorite_print",
    ];
    let derived_suffixes = [
        "_activity_sessions_v2",
        "_activity_bucket_cache_v2",
        "_activity_page_cache",
        "_activity_sync_state_v2",
        "_avatar_history",
        // v1-era caches + temporary stores: derivation-excluded, so they
        // exist only as PG residue and never sync down — expected.
        "_activity_cache_meta",
        "_activity_cache_sessions",
        "_activity_range_cache_v2",
        "_activity_top_worlds_cache_v",
        "_watched_users",
        "_mutual_graph_external_users",
        "_mutual_graph_manual_links",
    ];
    for (table, pg_count) in pg {
        let derived = derived_suffixes.iter().any(|s| table.ends_with(s));
        match sqlite.get(table.as_str()) {
            Some(sq_count) => {
                checked += 1;
                if pg_count != sq_count {
                    mismatches += 1;
                    findings.push(format!(
                        "count mismatch `{table}`: PG={pg_count} SQLite={sq_count} (delta {})",
                        pg_count - sq_count
                    ));
                }
            }
            None => {
                if derived || sqlite_only.iter().any(|t| *t == table.as_str()) {
                    continue;
                }
                findings.push(format!(
                    "table `{table}` exists in PG ({} rows) but is MISSING on SQLite side",
                    pg_count
                ));
            }
        }
    }
    println!("  compared {checked} shared tables, {mismatches} count mismatch(es)",);
}

//! One-off: check whether the 175 local-only mutual_graph_link ops ever
//! reached PG's sync_ops / materialized table.

use std::str::FromStr;

use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let dsn = "postgresql://vrcx:QS6NmP8MeHbQftPa@192.168.66.33:5432/vrcx?sslmode=prefer";
    let mut config = tokio_postgres::Config::from_str(dsn).unwrap();
    config.ssl_mode(tokio_postgres::config::SslMode::Prefer);
    let (client, connection) = config.connect(NoTls).await.expect("connect");
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let samples: Vec<(&str, &str)> = vec![(
        "usr_155ce704-8188-4fea-989f-26a629534b9c",
        "usr_3f8afbed-6290-4f0c-96b4-3b7e13536095",
    )];
    let table = "usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_links";
    for (friend, mutual) in samples {
        let key = format!(r#"[("{friend}"),("{mutual}")]"#);
        let key2 = format!(r#"["{friend}","{mutual}"]"#);
        for entity in [key.as_str(), key2.as_str()] {
            let rows = client
                .query(
                    "SELECT op_id, op, device, created_at, server_seq, hlc FROM sync_ops
                     WHERE table_name = $1 AND entity_key = $2 ORDER BY server_seq",
                    &[&table, &entity],
                )
                .await
                .unwrap_or_default();
            if !rows.is_empty() {
                for row in &rows {
                    let op_id: String = row.get(0);
                    let op: String = row.get(1);
                    let device: String = row.get(2);
                    let seq: i64 = row.get(4);
                    let hlc: String = row.get(5);
                    println!(
                        "  seq={seq} op={op} device={} hlc={}",
                        &device[..8.min(device.len())],
                        &hlc[..hlc.len().min(30)]
                    );
                    let _ = op_id;
                }
            }
        }
        // also check materialized row directly
        let mat = client
            .query_opt(
                "SELECT sync_device FROM usrdcf7bc5634d4482ab21ffb2c05dcfb2f_mutual_graph_links
                 WHERE friend_id = $1 AND mutual_id = $2",
                &[&friend, &mutual],
            )
            .await
            .unwrap();
        println!(
            "MATERIALIZED ({friend}..{mutual}..): {}",
            mat.map(|r| r.get::<_, String>(0))
                .unwrap_or_else(|| "ABSENT".into())
        );
    }

    // broader: how many ops exist for this table from server device
    let count = client
        .query_one(
            "SELECT count(*) FROM sync_ops WHERE table_name = $1 AND device = $2",
            &[&table, &"b00dfab1c35e41879145f39ec3147379".to_string()],
        )
        .await
        .unwrap();
    let c: i64 = count.get(0);
    println!("\nsync_ops from server device for links table: {c}");
    let any = client
        .query_one(
            "SELECT count(*) FROM sync_ops WHERE table_name = $1",
            &[&table],
        )
        .await
        .unwrap();
    let a: i64 = any.get(0);
    println!("sync_ops total for links table: {a}");
}

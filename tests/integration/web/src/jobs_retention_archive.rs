//! The weekly retention archive is written through core's file storage,
//! not the disk of the pod that ran the job. The ledger names the storage
//! object, the bytes behind it match the recorded SHA-256 after the job's own
//! exports directory is gone, and archives an older release left on a node's
//! disk are moved into storage (or dropped from the ledger when the node no
//! longer has them, so the period is archived again).

use chrono::{TimeZone, Utc};
use sha2::{Digest, Sha256};
use systemprompt::storage::{FileStorageBackend, build_file_storage};
use systemprompt::traits::StoredFileId;
use systemprompt_web_jobs::{RetentionExportWeeklyJob, WeeklyParams};

use crate::tempdb::TempDb;

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(out, "{b:02x}").expect("writing to a String cannot fail");
    }
    out
}

#[tokio::test]
async fn the_weekly_archive_lands_in_file_storage_and_the_ledger_names_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let storage_root = tempfile::tempdir().expect("storage root");
    let node_exports = tempfile::tempdir().expect("node exports");
    let storage = build_file_storage(FileStorageBackend::Local {
        root: storage_root.path().to_path_buf(),
    });
    let now = Utc.with_ymd_and_hms(2026, 9, 16, 3, 0, 0).unwrap();

    sqlx::query(
        "INSERT INTO retention_archives (tier, period, table_name, relative_path, row_count, \
         byte_count, sha256, window_from, window_to) VALUES \
         ('weekly', '2026-W30', 'ai_requests', 'weekly/2026-W30/ai_requests.jsonl.gz', 0, 3, 'x', now(), now()), \
         ('weekly', '2026-W31', 'ai_requests', 'weekly/2026-W31/ai_requests.jsonl.gz', 0, 3, 'x', now(), now())",
    )
    .execute(&*db.pool)
    .await
    .expect("seed legacy ledger rows");
    std::fs::create_dir_all(node_exports.path().join("weekly/2026-W30")).expect("legacy dir");
    std::fs::write(
        node_exports
            .path()
            .join("weekly/2026-W30/ai_requests.jsonl.gz"),
        b"old",
    )
    .expect("legacy file");

    RetentionExportWeeklyJob::execute_with_storage(
        &db.pool,
        storage.as_ref(),
        node_exports.path(),
        WeeklyParams {
            weeks_back: 1,
            keep_weeks: 520,
        },
        now,
    )
    .await
    .expect("weekly archive");
    drop(node_exports);

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT period, relative_path, sha256 FROM retention_archives \
         WHERE tier = 'weekly' ORDER BY period, table_name",
    )
    .fetch_all(&*db.pool)
    .await
    .expect("ledger");
    assert!(
        rows.iter()
            .all(|(_, path, _)| path.starts_with("exports/weekly/")),
        "every ledger row names a storage object: {rows:?}"
    );
    assert!(
        !rows.iter().any(|(period, _, _)| period == "2026-W31"),
        "a legacy period missing from this node is dropped to be re-archived"
    );
    let adopted = std::fs::read(
        storage_root
            .path()
            .join("exports/weekly/2026-W30/ai_requests.jsonl.gz"),
    )
    .expect("legacy archive moved into storage");
    assert_eq!(adopted, b"old");

    let fresh: Vec<&(String, String, String)> = rows
        .iter()
        .filter(|(period, _, _)| period == "2026-W37")
        .collect();
    assert!(!fresh.is_empty(), "last week was archived: {rows:?}");
    for (_, path, sha) in fresh {
        let body = std::fs::read(storage_root.path().join(path))
            .expect("archive readable from storage after the node's disk is gone");
        assert_eq!(&hex(&Sha256::digest(&body)), sha, "{path}");
    }
    assert!(
        storage
            .exists(&StoredFileId::new("exports/weekly/2026-W37/manifest.json"))
            .await
            .expect("manifest lookup"),
        "manifest stored"
    );
    db.cleanup().await;
}

//! Streams one table window to a gzipped JSON Lines object and records it.
//!
//! `SELECT row_to_json(t)::text` streamed a row at a time is the whole export
//! path: each row is written as one JSON object per line, gzipped as it
//! arrives, and a SHA-256 is taken over the compressed bytes so the manifest
//! can be verified without inflating it. Every file and the manifest are
//! stored through core's [`FileStorage`] under
//! `exports/<tier>/<period>/`, so a `gcs` backend (or a shared `local` root)
//! holds them rather than the disk of whichever pod ran the job. The ledger
//! rows for a period are written in one transaction only after the manifest
//! is stored, so a run that dies part-way leaves no ledger entry and the next
//! run archives the period again. Archives an older release recorded under
//! the node-local layout (`<tier>/<period>/<file>` under `storage/exports`)
//! are moved into storage by `adopt_local_archives`; a period with any file
//! missing on this node is dropped from the ledger so it is archived again
//! while its rows are still in the database. Period names are zero-padded
//! (`2026-W03`, `2026-03`), so pruning "before `keep_from`" is a string
//! compare.

use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use systemprompt::traits::{FileStorage, StoredFileId};
use systemprompt_web_admin_core::archive_objects::archive_object;

use super::ManagedTable;
use super::ledger::{
    ArchiveRecord, LedgerEntry, delete_period, legacy_entries, period_entries, record_archive,
    relocate_entry,
};
use crate::error::JobError;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Window {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ArchivedFile {
    pub table: &'static str,
    pub file: String,
    pub rows: i64,
    pub bytes: i64,
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct Manifest {
    pub tier: &'static str,
    pub period: String,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub written_at: DateTime<Utc>,
    pub files: Vec<ArchivedFile>,
}

pub(crate) struct ArchiveTarget<'a> {
    pub pool: &'a PgPool,
    pub storage: &'a dyn FileStorage,
    pub tier: &'static str,
    pub period: &'a str,
    pub window: Window,
}

pub(crate) async fn archive_tables(
    target: &ArchiveTarget<'_>,
    tables: &[ManagedTable],
) -> Result<Manifest, JobError> {
    let mut files = Vec::with_capacity(tables.len());
    for table in tables {
        files.push(archive_one(target, table).await?);
    }
    let manifest = Manifest {
        tier: target.tier,
        period: target.period.to_owned(),
        window_from: target.window.from,
        window_to: target.window.to,
        written_at: Utc::now(),
        files,
    };
    let body =
        serde_json::to_vec_pretty(&manifest).map_err(|error| JobError::other(error.to_string()))?;
    store(
        target.storage,
        &archive_object(target.tier, target.period, "manifest.json"),
        &body,
    )
    .await?;

    let mut tx = target.pool.begin().await?;
    for file in &manifest.files {
        record_archive(
            &mut tx,
            &ArchiveRecord {
                tier: target.tier,
                period: target.period,
                table: file.table,
                relative_path: &archive_object(target.tier, target.period, &file.file),
                row_count: file.rows,
                byte_count: file.bytes,
                sha256: &file.sha256,
                window_from: target.window.from,
                window_to: target.window.to,
            },
        )
        .await?;
    }
    tx.commit().await?;
    Ok(manifest)
}

async fn store(storage: &dyn FileStorage, object: &str, body: &[u8]) -> Result<(), JobError> {
    storage
        .store(Path::new(object), body)
        .await
        .map_err(|error| JobError::other(format!("store {object}: {error}")))?;
    Ok(())
}

async fn archive_one(
    target: &ArchiveTarget<'_>,
    table: &ManagedTable,
) -> Result<ArchivedFile, JobError> {
    let file_name = format!("{}.jsonl.gz", table.name);
    let mut stream = sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(select_statement(
        table,
        target.window,
    )))
    .fetch(target.pool);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut rows: i64 = 0;
    while let Some(row) = stream.next().await {
        let row = row?;
        encoder.write_all(row.as_bytes())?;
        encoder.write_all(b"\n")?;
        rows += 1;
    }
    drop(stream);
    let compressed = encoder.finish()?;
    let sha256 = hex::encode(Sha256::digest(&compressed));
    store(
        target.storage,
        &archive_object(target.tier, target.period, &file_name),
        &compressed,
    )
    .await?;
    Ok(ArchivedFile {
        table: table.name,
        file: file_name,
        rows,
        bytes: i64::try_from(compressed.len()).unwrap_or(i64::MAX),
        sha256,
    })
}

// Why: Postgres `COPY … TO STDOUT` text format escapes backslashes, so a JSON
// string containing \" arrives as \\", which is invalid JSON;
// a streamed `SELECT` returns exactly what `row_to_json` produced.
fn select_statement(table: &ManagedTable, window: Window) -> String {
    format!(
        "SELECT row_to_json(t)::text FROM {name} t \
         WHERE {col} >= '{from}'::timestamptz AND {col} < '{to}'::timestamptz \
         ORDER BY {col}",
        name = table.name,
        col = table.time_column,
        from = window.from.to_rfc3339(),
        to = window.to.to_rfc3339(),
    )
}

pub(crate) async fn prune_periods(
    pool: &PgPool,
    storage: &dyn FileStorage,
    tier: &str,
    keep_from: &str,
) -> Result<u64, JobError> {
    let mut removed = 0;
    for (period, entries) in group_by_period(period_entries(pool, tier, keep_from).await?) {
        for entry in &entries {
            remove(storage, &entry.relative_path).await?;
        }
        remove(storage, &archive_object(tier, &period, "manifest.json")).await?;
        delete_period(pool, tier, &period).await?;
        removed += 1;
    }
    Ok(removed)
}

async fn remove(storage: &dyn FileStorage, object: &str) -> Result<(), JobError> {
    match storage.delete(&StoredFileId::new(object)).await {
        Ok(()) | Err(systemprompt::traits::FileStorageError::NotFound(_)) => Ok(()),
        Err(error) => Err(JobError::other(format!("delete {object}: {error}"))),
    }
}

pub(crate) async fn adopt_local_archives(
    pool: &PgPool,
    storage: &dyn FileStorage,
    exports_root: &Path,
    tier: &str,
) -> Result<(u64, u64), JobError> {
    let mut adopted = 0;
    let mut dropped = 0;
    for (period, entries) in group_by_period(legacy_entries(pool, tier).await?) {
        let mut bodies = Vec::with_capacity(entries.len());
        for entry in &entries {
            match std::fs::read(exports_root.join(&entry.relative_path)) {
                Ok(body) => bodies.push(body),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(error.into()),
            }
        }
        if bodies.len() < entries.len() {
            tracing::warn!(
                tier,
                period,
                "archive missing on this node; dropped from the ledger to be re-archived"
            );
            delete_period(pool, tier, &period).await?;
            dropped += 1;
            continue;
        }
        for (entry, body) in entries.iter().zip(&bodies) {
            store(
                storage,
                &format!(
                    "{}/{}",
                    systemprompt_web_admin_core::archive_objects::ARCHIVE_PREFIX,
                    entry.relative_path
                ),
                body,
            )
            .await?;
        }
        if let Ok(manifest) =
            std::fs::read(exports_root.join(tier).join(&period).join("manifest.json"))
        {
            store(
                storage,
                &archive_object(tier, &period, "manifest.json"),
                &manifest,
            )
            .await?;
        }
        let mut tx = pool.begin().await?;
        for entry in &entries {
            relocate_entry(&mut tx, entry).await?;
        }
        tx.commit().await?;
        adopted += 1;
    }
    Ok((adopted, dropped))
}

fn group_by_period(entries: Vec<LedgerEntry>) -> Vec<(String, Vec<LedgerEntry>)> {
    let mut grouped: Vec<(String, Vec<LedgerEntry>)> = Vec::new();
    for entry in entries {
        match grouped.last_mut() {
            Some((period, list)) if *period == entry.period => list.push(entry),
            _ => grouped.push((entry.period.clone(), vec![entry])),
        }
    }
    grouped
}

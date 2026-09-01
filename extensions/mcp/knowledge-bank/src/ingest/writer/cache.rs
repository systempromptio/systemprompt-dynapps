//! The two cost caches (NFR-COST-1) and the per-source, per-user sync cursor.
//!
//! Embeddings are keyed by model and text hash, so a rebuild or a re-chunked
//! document only pays for text never embedded before. LLM-derived dates are
//! frozen per content hash, so an unchanged document never re-asks the model
//! — the same role the Node system's committed `.dates.json` played.

use super::ResolvedDate;
use crate::error::KnowledgeBankError;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[must_use]
pub fn text_hash(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

pub async fn find_embedding(
    pool: &PgPool,
    embed_model: &str,
    hash: &str,
) -> Result<Option<String>, KnowledgeBankError> {
    let literal = sqlx::query_scalar!(
        r#"SELECT embedding::text AS "embedding!" FROM knowledge_embedding_cache
           WHERE embed_model = $1 AND text_hash = $2"#,
        embed_model,
        hash,
    )
    .fetch_optional(pool)
    .await?;
    Ok(literal)
}

pub async fn store_embedding(
    pool: &PgPool,
    embed_model: &str,
    hash: &str,
    literal: &str,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"INSERT INTO knowledge_embedding_cache (embed_model, text_hash, embedding)
           VALUES ($1, $2, ($3::text)::vector)
           ON CONFLICT (embed_model, text_hash) DO NOTHING"#,
        embed_model,
        hash,
        literal,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_date(
    pool: &PgPool,
    doc_id: &str,
    content_hash: &str,
) -> Result<Option<ResolvedDate>, KnowledgeBankError> {
    let row = sqlx::query!(
        r#"SELECT doc_date, date_source FROM knowledge_date_cache
           WHERE doc_id = $1 AND content_hash = $2"#,
        doc_id,
        content_hash,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| ResolvedDate {
        date: r.doc_date,
        source: super::super::dating::source_label(&r.date_source),
    }))
}

pub async fn store_date(
    pool: &PgPool,
    doc_id: &str,
    content_hash: &str,
    date: ResolvedDate,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"INSERT INTO knowledge_date_cache (doc_id, content_hash, doc_date, date_source, resolved_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (doc_id) DO UPDATE SET
               content_hash = EXCLUDED.content_hash,
               doc_date = EXCLUDED.doc_date,
               date_source = EXCLUDED.date_source,
               resolved_at = NOW()"#,
        doc_id,
        content_hash,
        date.date,
        date.source,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn read_last_sync(
    pool: &PgPool,
    source_type: &str,
    user_id: &UserId,
) -> Result<Option<DateTime<Utc>>, KnowledgeBankError> {
    let row = sqlx::query!(
        r#"SELECT last_sync FROM knowledge_user_sync_state
           WHERE source_type = $1 AND user_id = $2"#,
        source_type,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.last_sync))
}

/// Counters for one source in one run.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncCounters {
    pub seen: i32,
    pub written: i32,
    pub pruned: i32,
    pub failed: i32,
}

/// Which caller's cursor a sync-state write belongs to.
#[derive(Debug, Clone, Copy)]
pub struct SyncKey<'a> {
    pub source_type: &'a str,
    pub user_id: &'a UserId,
}

pub async fn write_sync_state(
    pool: &PgPool,
    key: SyncKey<'_>,
    started_at: DateTime<Utc>,
    counters: SyncCounters,
    error: Option<&str>,
) -> Result<(), KnowledgeBankError> {
    // Why: the cursor advances only on a clean run. A failed run keeps the old
    // cursor so the next attempt re-reads whatever the failure may have missed.
    let last_sync = if error.is_none() {
        Some(started_at)
    } else {
        None
    };
    sqlx::query!(
        r#"INSERT INTO knowledge_user_sync_state
               (source_type, user_id, last_sync, last_run_at, last_error,
                documents_seen, documents_written, documents_pruned, documents_failed)
           VALUES ($1, $2, $3, NOW(), $4, $5, $6, $7, $8)
           ON CONFLICT (source_type, user_id) DO UPDATE SET
               last_sync = COALESCE(EXCLUDED.last_sync, knowledge_user_sync_state.last_sync),
               last_run_at = NOW(),
               last_error = EXCLUDED.last_error,
               documents_seen = EXCLUDED.documents_seen,
               documents_written = EXCLUDED.documents_written,
               documents_pruned = EXCLUDED.documents_pruned,
               documents_failed = EXCLUDED.documents_failed"#,
        key.source_type,
        key.user_id.as_str(),
        last_sync,
        error,
        counters.seen,
        counters.written,
        counters.pruned,
        counters.failed,
    )
    .execute(pool)
    .await?;
    Ok(())
}

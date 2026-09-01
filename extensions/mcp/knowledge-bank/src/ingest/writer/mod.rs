//! Index writes: documents, their chunks, their grants, pruning, and the
//! version stamp.
//!
//! Upsert is delete-then-insert per document inside one transaction, so a
//! reader never sees half a document. Grants are rewritten wholesale on every
//! write, which is what makes REQ-052's "permission changes propagate" true:
//! a permission dropped in the source system disappears from the index at the
//! next sync, with no separate revocation path to forget to call.

pub mod cache;

use super::document::SourceDocument;
use crate::error::KnowledgeBankError;
use crate::principal::Subject;
use chrono::NaiveDate;
use sqlx::PgPool;

/// A chunk ready to be stored: the embedded text and its metadata.
#[derive(Debug, Clone)]
pub struct StoredChunk {
    pub text: String,
    pub breadcrumb: String,
    pub lines: Option<String>,
    pub embedding: String,
}

/// The resolved date and its provenance, written beside the document.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedDate {
    pub date: Option<NaiveDate>,
    pub source: &'static str,
}

pub async fn find_content_hash(
    pool: &PgPool,
    doc_id: &str,
) -> Result<Option<String>, KnowledgeBankError> {
    let existing = sqlx::query_scalar!(
        r#"SELECT content_hash FROM knowledge_documents WHERE doc_id = $1"#,
        doc_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(existing)
}

pub async fn upsert_document(
    pool: &PgPool,
    document: &SourceDocument,
    date: ResolvedDate,
    chunks: &[StoredChunk],
    archived: bool,
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO knowledge_documents
            (doc_id, source_type, source, source_ref, title, uri, doc_date, date_source,
             content_hash, mtime, indexed_at, archived)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), $11)
        ON CONFLICT (doc_id) DO UPDATE SET
            source_type = EXCLUDED.source_type,
            source = EXCLUDED.source,
            source_ref = EXCLUDED.source_ref,
            title = EXCLUDED.title,
            uri = EXCLUDED.uri,
            doc_date = EXCLUDED.doc_date,
            date_source = EXCLUDED.date_source,
            content_hash = EXCLUDED.content_hash,
            mtime = EXCLUDED.mtime,
            indexed_at = EXCLUDED.indexed_at,
            archived = EXCLUDED.archived
        "#,
        document.doc_id,
        document.source_type,
        document.source,
        document.source_ref,
        document.title,
        document.uri,
        date.date,
        date.source,
        document.content_hash,
        document.mtime,
        archived,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"DELETE FROM knowledge_chunks WHERE doc_id = $1"#,
        document.doc_id
    )
    .execute(&mut *tx)
    .await?;

    let total = i32::try_from(chunks.len())
        // Why: TryFromIntError is variant-less.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("chunk count: {e}")))?;
    for (index, chunk) in chunks.iter().enumerate() {
        let position = i32::try_from(index)
            // Why: TryFromIntError is variant-less.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::Internal(format!("chunk index: {e}")))?;
        let chunk_id = format!("{}#{index}", document.doc_id);
        sqlx::query!(
            r#"
            INSERT INTO knowledge_chunks
                (chunk_id, doc_id, text, embedding, breadcrumb, lines, chunk_index, chunk_total)
            VALUES ($1, $2, $3, ($4::text)::vector, $5, $6, $7, $8)
            "#,
            chunk_id,
            document.doc_id,
            chunk.text,
            chunk.embedding,
            chunk.breadcrumb,
            chunk.lines,
            position,
            total,
        )
        .execute(&mut *tx)
        .await?;
    }

    write_grants(&mut tx, &document.doc_id, &document.grants).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn rewrite_grants(
    pool: &PgPool,
    doc_id: &str,
    grants: &[Subject],
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;
    write_grants(&mut tx, doc_id, grants).await?;
    tx.commit().await?;
    Ok(())
}

// Why: grants are rewritten wholesale rather than merged. A merge would keep
// a permission the source system has since removed, which is exactly the
// propagation failure REQ-052 forbids.
async fn write_grants(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    doc_id: &str,
    grants: &[Subject],
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"DELETE FROM knowledge_document_grants WHERE doc_id = $1"#,
        doc_id
    )
    .execute(&mut **tx)
    .await?;
    for grant in grants {
        sqlx::query!(
            r#"
            INSERT INTO knowledge_document_grants (doc_id, subject_kind, subject_value)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            doc_id,
            grant.kind,
            grant.value,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn prune_missing(
    pool: &PgPool,
    source_type: &str,
    current_refs: &[String],
) -> Result<u64, KnowledgeBankError> {
    let result = sqlx::query!(
        r#"DELETE FROM knowledge_documents WHERE source_type = $1 AND NOT (source_ref = ANY($2))"#,
        source_type,
        current_refs,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn delete_source(pool: &PgPool, source_type: &str) -> Result<u64, KnowledgeBankError> {
    let result = sqlx::query!(
        r#"DELETE FROM knowledge_documents WHERE source_type = $1"#,
        source_type
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn stamp_index_state(pool: &PgPool) -> Result<String, KnowledgeBankError> {
    let latest =
        sqlx::query_scalar!(r#"SELECT MAX(indexed_at) AS "latest?" FROM knowledge_documents"#)
            .fetch_one(pool)
            .await?;
    let version = latest.map_or_else(|| "empty".to_owned(), |t| t.to_rfc3339());
    let now = chrono::Utc::now().to_rfc3339();
    for (key, value) in [("version", version.as_str()), ("last_built", now.as_str())] {
        sqlx::query!(
            r#"
            INSERT INTO knowledge_index_state (key, value, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()
            "#,
            key,
            value,
        )
        .execute(pool)
        .await?;
    }
    Ok(version)
}

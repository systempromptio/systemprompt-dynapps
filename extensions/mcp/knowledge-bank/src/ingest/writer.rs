//! Index writes: documents, their chunks, and their grants.
//!
//! Upsert is delete-then-insert per document, keyed on `content_hash` and
//! `mtime` so an unchanged document is skipped entirely — the same
//! incremental-sync contract as the Node system's `getIndexedDocs`.
//!
//! Grants are rewritten on every sync of a document, which is what makes
//! REQ-052's "permission changes propagate" true: a permission dropped in the
//! source system disappears from the index at the next sync, with no separate
//! revocation path to forget to call.

use crate::error::KnowledgeBankError;
use sqlx::PgPool;

/// A source document as ingestion sees it, before chunking.
#[derive(Debug, Clone)]
pub struct SourceDocument {
    pub doc_id: String,
    pub source_type: String,
    pub source: String,
    pub title: String,
    pub uri: String,
    pub body: String,
    pub content_hash: String,
    pub doc_date: Option<chrono::DateTime<chrono::Utc>>,
    pub date_source: Option<String>,
    pub grants: Vec<(String, String)>,
}

pub async fn is_unchanged(
    pool: &PgPool,
    doc_id: &str,
    content_hash: &str,
) -> Result<bool, KnowledgeBankError> {
    let existing = sqlx::query_scalar!(
        r#"SELECT content_hash FROM knowledge_documents WHERE doc_id = $1"#,
        doc_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(existing.as_deref() == Some(content_hash))
}

pub async fn upsert_document(
    pool: &PgPool,
    document: &SourceDocument,
    chunks: &[(String, String)],
    embeddings: &[String],
) -> Result<(), KnowledgeBankError> {
    let mut tx = pool.begin().await?;

    write_document_row(&mut tx, document).await?;
    write_chunks(&mut tx, document, chunks, embeddings).await?;
    write_grants(&mut tx, document).await?;

    tx.commit().await?;
    Ok(())
}

async fn write_document_row(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document: &SourceDocument,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"
        INSERT INTO knowledge_documents
            (doc_id, source_type, source, title, uri, doc_date, date_source,
             content_hash, mtime, indexed_at, archived)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW(), FALSE)
        ON CONFLICT (doc_id) DO UPDATE SET
            source_type = EXCLUDED.source_type,
            source = EXCLUDED.source,
            title = EXCLUDED.title,
            uri = EXCLUDED.uri,
            doc_date = EXCLUDED.doc_date,
            date_source = EXCLUDED.date_source,
            content_hash = EXCLUDED.content_hash,
            mtime = EXCLUDED.mtime,
            indexed_at = EXCLUDED.indexed_at
        "#,
        document.doc_id,
        document.source_type,
        document.source,
        document.title,
        document.uri,
        document.doc_date,
        document.date_source,
        document.content_hash,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn write_chunks(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document: &SourceDocument,
    chunks: &[(String, String)],
    embeddings: &[String],
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"DELETE FROM knowledge_chunks WHERE doc_id = $1"#,
        document.doc_id
    )
    .execute(&mut **tx)
    .await?;

    let total = i32::try_from(chunks.len())
        // Why: TryFromIntError is variant-less.
        // lint-ok: error-adapt
        .map_err(|e| KnowledgeBankError::Internal(format!("chunk count: {e}")))?;
    for (index, ((text, lines), embedding)) in chunks.iter().zip(embeddings).enumerate() {
        let position = i32::try_from(index)
            // Why: TryFromIntError is variant-less.
            // lint-ok: error-adapt
            .map_err(|e| KnowledgeBankError::Internal(format!("chunk index: {e}")))?;
        let chunk_id = format!("{}#{index}", document.doc_id);
        let lines = (!lines.is_empty()).then(|| lines.clone());
        sqlx::query!(
            r#"
            INSERT INTO knowledge_chunks
                (chunk_id, doc_id, text, embedding, lines, chunk_index, chunk_total)
            VALUES ($1, $2, $3, ($4::text)::vector, $5, $6, $7)
            "#,
            chunk_id,
            document.doc_id,
            text,
            embedding,
            lines,
            position,
            total,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

// Why: grants are rewritten wholesale rather than merged. A merge would keep
// a permission the source system has since removed, which is exactly the
// propagation failure REQ-052 forbids.
async fn write_grants(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    document: &SourceDocument,
) -> Result<(), KnowledgeBankError> {
    sqlx::query!(
        r#"DELETE FROM knowledge_document_grants WHERE doc_id = $1"#,
        document.doc_id
    )
    .execute(&mut **tx)
    .await?;

    for (kind, value) in &document.grants {
        sqlx::query!(
            r#"
            INSERT INTO knowledge_document_grants (doc_id, subject_kind, subject_value)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            document.doc_id,
            kind,
            value,
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub async fn stamp_index_state(
    pool: &PgPool,
    version: &str,
    last_built: &str,
) -> Result<(), KnowledgeBankError> {
    for (key, value) in [("version", version), ("last_built", last_built)] {
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
    Ok(())
}

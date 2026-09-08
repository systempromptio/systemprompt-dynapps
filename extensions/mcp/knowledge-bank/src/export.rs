//! Complete, caller-scoped source exports for requirements evidence snapshots.
use crate::error::KnowledgeBankError;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceExport {
    pub source_ref: String,
    pub source_type: String,
    pub title: String,
    pub uri: String,
    pub body: String,
    pub content_hash: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceSyncState {
    pub source_type: String,
    pub last_run: Option<String>,
    pub error: Option<String>,
    pub documents_seen: i32,
    pub documents_failed: i32,
}

pub async fn list_sync_state(
    pool: &PgPool,
    user: &UserId,
) -> Result<Vec<SourceSyncState>, KnowledgeBankError> {
    let rows=sqlx::query("SELECT source_type,last_run_at::text AS last_run,last_error,documents_seen,documents_failed FROM knowledge_user_sync_state WHERE user_id=$1 ORDER BY source_type")
        .bind(user.as_str()).fetch_all(pool).await?;
    rows.iter()
        .map(|r| {
            Ok(SourceSyncState {
                source_type: r.try_get("source_type")?,
                last_run: r.try_get("last_run")?,
                error: r.try_get("last_error")?,
                documents_seen: r.try_get("documents_seen")?,
                documents_failed: r.try_get("documents_failed")?,
            })
        })
        .collect()
}

pub async fn export_sources(
    pool: &PgPool,
    user: &UserId,
    after: &str,
    limit: i64,
) -> Result<Vec<SourceExport>, KnowledgeBankError> {
    if !(1..=100).contains(&limit) {
        return Err(KnowledgeBankError::InvalidInput(
            "limit must be 1–100".to_owned(),
        ));
    }
    let rows=sqlx::query("SELECT d.doc_id,d.source_ref,d.source_type,d.title,d.uri,t.body,d.content_hash,COALESCE(d.mtime,d.indexed_at)::text AS version FROM knowledge_documents d JOIN knowledge_source_text t ON t.doc_id=d.doc_id AND t.content_hash=d.content_hash WHERE d.doc_id>$2 AND d.source_type IN ('jira','confluence') AND EXISTS(SELECT 1 FROM knowledge_document_grants g WHERE g.doc_id=d.doc_id AND g.subject_kind='user' AND g.subject_value=$1) ORDER BY d.doc_id LIMIT $3")
        .bind(user.as_str()).bind(after).bind(limit).fetch_all(pool).await?;
    rows.iter()
        .map(|r| {
            Ok(SourceExport {
                source_ref: r.try_get("source_ref")?,
                source_type: r.try_get("source_type")?,
                title: r.try_get("title")?,
                uri: r.try_get("uri")?,
                body: r.try_get("body")?,
                content_hash: r.try_get("content_hash")?,
                version: r.try_get("version")?,
            })
        })
        .collect()
}

pub async fn latest_run(
    pool: &PgPool,
    user: &UserId,
) -> Result<Option<serde_json::Value>, KnowledgeBankError> {
    Ok(sqlx::query_scalar("SELECT to_jsonb(r) FROM knowledge_sync_runs r WHERE user_id=$1 ORDER BY started_at DESC,run_id DESC LIMIT 1")
        .bind(user.as_str()).fetch_optional(pool).await?)
}

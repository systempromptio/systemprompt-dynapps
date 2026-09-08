//! Complete, caller-scoped source exports for requirements evidence snapshots.
use crate::error::KnowledgeBankError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    Ok(sqlx::query_as!(SourceSyncState,"SELECT source_type,last_run_at::text AS last_run,last_error AS error,documents_seen,documents_failed FROM knowledge_user_sync_state WHERE user_id=$1 ORDER BY source_type",user.as_str()).fetch_all(pool).await?)
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
    Ok(sqlx::query_as!(SourceExport,"SELECT d.source_ref,d.source_type,d.title,d.uri,t.body,d.content_hash,COALESCE(d.mtime,d.indexed_at)::text AS \"version!\" FROM knowledge_documents d JOIN knowledge_source_text t ON t.doc_id=d.doc_id AND t.content_hash=d.content_hash WHERE d.doc_id>$2 AND d.source_type IN ('jira','confluence') AND EXISTS(SELECT 1 FROM knowledge_document_grants g WHERE g.doc_id=d.doc_id AND g.subject_kind='user' AND g.subject_value=$1) ORDER BY d.doc_id LIMIT $3",user.as_str(),after,limit).fetch_all(pool).await?)
}

// JSON: the latest run is returned as the row's own JSON projection so the
// requirements evidence snapshot carries whatever columns the run recorded.
pub async fn latest_run(
    pool: &PgPool,
    user: &UserId,
) -> Result<Option<serde_json::Value>, KnowledgeBankError> {
    Ok(sqlx::query_scalar!("SELECT to_jsonb(r) AS \"run!\" FROM knowledge_sync_runs r WHERE user_id=$1 ORDER BY started_at DESC,run_id DESC LIMIT 1",user.as_str()).fetch_optional(pool).await?)
}

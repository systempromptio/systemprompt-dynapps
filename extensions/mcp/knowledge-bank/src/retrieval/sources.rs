//! Source listing and index statistics.
//!
//! `list_sources` is access-controlled too: every configured category is
//! listed — the catalogue is configuration, not content — but the document
//! count is the *caller's* count and the last sync is the caller's own.
//! Reporting the true totals would tell an unauthorized caller how much
//! material exists in a category they cannot read, which is the disclosure
//! REQ-052 is meant to prevent.

use crate::config::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;
use crate::principal::Principal;
use crate::retriever::{IndexStats, KnowledgeSource};
use sqlx::PgPool;
use std::collections::HashMap;

pub(super) const STATE_KEY_VERSION: &str = "version";
pub(super) const STATE_KEY_LAST_BUILT: &str = "last_built";

pub(super) async fn list_visible_sources(
    pool: &PgPool,
    config: &KnowledgeBankConfig,
    principal: &Principal,
) -> Result<Vec<KnowledgeSource>, KnowledgeBankError> {
    let subjects = principal.subjects();
    let kinds: Vec<String> = subjects.iter().map(|s| s.kind.clone()).collect();
    let values: Vec<String> = subjects.iter().map(|s| s.value.clone()).collect();

    let counts = sqlx::query!(
        r#"
        SELECT d.source_type AS "source_type!", COUNT(*) AS "doc_count!"
        FROM knowledge_documents d
        WHERE EXISTS (
            SELECT 1 FROM knowledge_document_grants g
            WHERE g.doc_id = d.doc_id
              AND (g.subject_kind, g.subject_value) IN (
                  SELECT * FROM UNNEST($1::text[], $2::text[])
              )
        )
        GROUP BY d.source_type
        "#,
        &kinds,
        &values,
    )
    .fetch_all(pool)
    .await?;
    let counts: HashMap<String, i64> = counts
        .into_iter()
        .map(|row| (row.source_type, row.doc_count))
        .collect();

    let synced = sqlx::query!(
        r#"SELECT source_type, last_run_at, last_error FROM knowledge_user_sync_state
           WHERE user_id = $1"#,
        principal.user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;
    // Why: a clean run reports when it ran; a failed or skipped one reports
    // why, so the caller learns that Atlassian was left out and how to fix it.
    let synced: HashMap<String, (Option<chrono::DateTime<chrono::Utc>>, Option<String>)> = synced
        .into_iter()
        .map(|row| {
            let state = match row.last_error {
                Some(error) => (None, Some(error)),
                None => (row.last_run_at, None),
            };
            (row.source_type, state)
        })
        .collect();

    Ok(config
        .sources
        .iter()
        .map(|(id, source)| {
            let doc_count = counts.get(id).copied().unwrap_or(0);
            KnowledgeSource {
                id: id.clone(),
                name: id.replace('_', " "),
                description: source.description.clone(),
                archived: source.archived,
                available: doc_count > 0,
                doc_count: u64::try_from(doc_count).ok(),
                last_synced: synced
                    .get(id)
                    .and_then(|(t, _)| t.as_ref())
                    .map(chrono::DateTime::to_rfc3339),
                last_error: synced.get(id).and_then(|(_, e)| e.clone()),
            }
        })
        .collect())
}

pub(super) async fn read_index_stats(pool: &PgPool) -> Result<IndexStats, KnowledgeBankError> {
    let totals = sqlx::query!(
        r#"
        SELECT (SELECT COUNT(*) FROM knowledge_documents) AS "documents!",
               (SELECT COUNT(*) FROM knowledge_chunks) AS "chunks!",
               (SELECT COUNT(*) FROM knowledge_document_grants) AS "grants!",
               (SELECT COUNT(DISTINCT user_id) FROM knowledge_user_sync_state) AS "syncing_users!",
               (SELECT COUNT(*) FROM knowledge_embedding_cache) AS "embedding_cache_rows!",
               pg_database_size(current_database()) AS "database_bytes!"
        "#
    )
    .fetch_one(pool)
    .await?;

    let state = sqlx::query!(
        r#"SELECT key, value FROM knowledge_index_state WHERE key = ANY($1)"#,
        &[
            STATE_KEY_VERSION.to_owned(),
            STATE_KEY_LAST_BUILT.to_owned()
        ][..],
    )
    .fetch_all(pool)
    .await?;

    let find = |wanted: &str| {
        state
            .iter()
            .find(|row| row.key == wanted)
            .map(|row| row.value.clone())
    };

    Ok(IndexStats {
        documents: u64::try_from(totals.documents).unwrap_or(0),
        chunks: u64::try_from(totals.chunks).unwrap_or(0),
        grants: u64::try_from(totals.grants).unwrap_or(0),
        syncing_users: u64::try_from(totals.syncing_users).unwrap_or(0),
        embedding_cache_rows: u64::try_from(totals.embedding_cache_rows).unwrap_or(0),
        database_bytes: u64::try_from(totals.database_bytes).unwrap_or(0),
        last_built: find(STATE_KEY_LAST_BUILT),
        version: find(STATE_KEY_VERSION),
    })
}

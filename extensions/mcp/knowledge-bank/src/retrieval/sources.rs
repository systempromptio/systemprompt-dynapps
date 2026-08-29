//! Source listing and index statistics.
//!
//! `list_sources` is access-controlled too: a caller sees a source only when
//! at least one document in it is granted to them, and the document count is
//! their count, not the index's. Reporting the true totals would tell an
//! unauthorized caller how much material exists in a category they cannot
//! read, which is the disclosure REQ-052 is meant to prevent.

use crate::config::KnowledgeBankConfig;
use crate::error::KnowledgeBankError;
use crate::principal::Principal;
use crate::retriever::{IndexStats, KnowledgeSource};
use sqlx::PgPool;

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

    let rows = sqlx::query!(
        r#"
        SELECT d.source_type AS "source_type!",
               COUNT(*) AS "doc_count!",
               MAX(d.indexed_at) AS "last_synced?"
        FROM knowledge_documents d
        WHERE EXISTS (
            SELECT 1 FROM knowledge_document_grants g
            WHERE g.doc_id = d.doc_id
              AND (g.subject_kind, g.subject_value) IN (
                  SELECT * FROM UNNEST($1::text[], $2::text[])
              )
        )
        GROUP BY d.source_type
        ORDER BY d.source_type
        "#,
        &kinds,
        &values,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let configured = config.sources.get(&row.source_type);
            KnowledgeSource {
                id: row.source_type.clone(),
                name: row.source_type.clone(),
                description: configured
                    .map(|c| c.description.clone())
                    .unwrap_or_default(),
                available: true,
                doc_count: u64::try_from(row.doc_count).ok(),
                last_synced: row.last_synced.map(|t| t.to_rfc3339()),
            }
        })
        .collect())
}

pub(super) async fn read_index_stats(pool: &PgPool) -> Result<IndexStats, KnowledgeBankError> {
    let totals = sqlx::query!(
        r#"
        SELECT (SELECT COUNT(*) FROM knowledge_documents) AS "documents!",
               (SELECT COUNT(*) FROM knowledge_chunks) AS "chunks!"
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
        last_built: find(STATE_KEY_LAST_BUILT),
        version: find(STATE_KEY_VERSION),
    })
}

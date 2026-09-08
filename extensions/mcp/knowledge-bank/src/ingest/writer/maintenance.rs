//! Index maintenance after a write pass: orphan pruning and the version stamp.

use crate::error::KnowledgeBankError;
use sqlx::PgPool;

pub async fn delete_orphan_documents(pool: &PgPool) -> Result<u64, KnowledgeBankError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM knowledge_documents d
        WHERE NOT EXISTS (
            SELECT 1 FROM knowledge_document_grants g WHERE g.doc_id = d.doc_id
        )
        "#
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

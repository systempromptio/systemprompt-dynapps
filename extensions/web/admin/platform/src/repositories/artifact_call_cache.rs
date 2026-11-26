//! A live page's cached reads, per viewer, document and call (the input
//! hashed with its keys sorted). Only a read's successful result is stored,
//! never a write's; an entry answers until `expires_at`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
// JSON: MCP-protocol boundary, a tool result is schema-less.
use serde_json::Value;
use systemprompt::identifiers::{DocumentId, UserId};

#[derive(Debug)]
pub struct CachedCallKey<'a> {
    pub user: &'a UserId,
    pub document: &'a DocumentId,
    pub server: &'a str,
    pub tool: &'a str,
    pub input_hash: &'a str,
}

// JSON: MCP-protocol boundary, the stored tool result.
#[derive(Debug)]
pub struct CachedCall {
    pub result: Value,
    pub stored_at: DateTime<Utc>,
}

/// Which of a viewer's entries on a document to drop: all of them, one
/// server's, one tool's, or one call's.
#[derive(Debug)]
pub struct CachedCallScope<'a> {
    pub user: &'a UserId,
    pub document: &'a DocumentId,
    pub server: Option<&'a str>,
    pub tool: Option<&'a str>,
    pub input_hash: Option<&'a str>,
}

pub async fn find_artifact_cached_call(
    pool: &PgPool,
    key: &CachedCallKey<'_>,
) -> Result<Option<CachedCall>, sqlx::Error> {
    sqlx::query_as!(
        CachedCall,
        r#"SELECT result, stored_at
             FROM artifact_call_cache
            WHERE user_id = $1 AND document_id = $2::text::uuid
              AND server = $3 AND tool = $4 AND input_hash = $5
              AND expires_at > CURRENT_TIMESTAMP"#,
        key.user.as_str(),
        key.document.as_str(),
        key.server,
        key.tool,
        key.input_hash,
    )
    .fetch_optional(pool)
    .await
}

// JSON: MCP-protocol boundary, the tool result to store.
pub async fn set_artifact_cached_call(
    pool: &PgPool,
    key: &CachedCallKey<'_>,
    result: &Value,
    expires_at: DateTime<Utc>,
) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"WITH expired AS (
               DELETE FROM artifact_call_cache
                WHERE user_id = $1 AND expires_at <= CURRENT_TIMESTAMP
                  AND NOT (document_id = $2::text::uuid AND server = $3
                           AND tool = $4 AND input_hash = $5)
           )
           INSERT INTO artifact_call_cache
                (user_id, document_id, server, tool, input_hash, result, stored_at, expires_at)
           VALUES ($1, $2::text::uuid, $3, $4, $5, $6, CURRENT_TIMESTAMP, $7)
           ON CONFLICT (user_id, document_id, server, tool, input_hash) DO UPDATE
              SET result = EXCLUDED.result,
                  stored_at = EXCLUDED.stored_at,
                  expires_at = EXCLUDED.expires_at
           RETURNING stored_at"#,
        key.user.as_str(),
        key.document.as_str(),
        key.server,
        key.tool,
        key.input_hash,
        result,
        expires_at,
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_artifact_cached_calls(
    pool: &PgPool,
    scope: &CachedCallScope<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM artifact_call_cache
            WHERE user_id = $1 AND document_id = $2::text::uuid
              AND ($3::text IS NULL OR server = $3)
              AND ($4::text IS NULL OR tool = $4)
              AND ($5::text IS NULL OR input_hash = $5)"#,
        scope.user.as_str(),
        scope.document.as_str(),
        scope.server,
        scope.tool,
        scope.input_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}

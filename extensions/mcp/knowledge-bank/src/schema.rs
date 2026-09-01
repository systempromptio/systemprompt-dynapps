//! Schema ownership for the knowledge bank.
//!
//! The tables need pgvector, and the platform's schema inventory installs
//! everything at boot or fails the whole boot — so an instance whose Postgres
//! lacks the extension (the Fly staging deploy on the managed database) would
//! crash-loop over an optional MCP server. The binary therefore probes for
//! the extension itself and applies its own idempotent schema only where it
//! can; everywhere else it serves the honest "not configured" stub.

use crate::error::KnowledgeBankError;
use sqlx::PgPool;

const SCHEMA: &str = include_str!("../schema/knowledge_bank.sql");

pub async fn pgvector_available(pool: &PgPool) -> Result<bool, KnowledgeBankError> {
    let available = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM pg_available_extensions WHERE name = 'vector'
           ) AS "available!""#
    )
    .fetch_one(pool)
    .await?;
    Ok(available)
}

pub async fn apply(pool: &PgPool) -> Result<(), KnowledgeBankError> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}

pub async fn ensure(pool: &PgPool) -> Result<bool, KnowledgeBankError> {
    if !pgvector_available(pool).await? {
        return Ok(false);
    }
    apply(pool).await?;
    Ok(true)
}

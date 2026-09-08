//! Schema ownership for the knowledge bank.
//!
//! The index lives in its own pgvector database (or the instance database
//! when no `knowledge_database_url` secret is set), so the platform's schema
//! inventory never sees these tables: the binary applies its own idempotent
//! schema at boot. pgvector is a hard requirement — a database without it
//! fails this process, and with it the platform's boot, with a message that
//! names the database, because an index that silently cannot exist is worse
//! than a deploy that stops.

use crate::error::KnowledgeBankError;
use sqlx::PgPool;

const SCHEMA: &str = include_str!("../schema/knowledge_bank.sql");

// Why: `CREATE TABLE IF NOT EXISTS` cannot reshape a table that already
// exists, and scripts/lint-schema.sh keeps ALTER/DROP out of schema/*.sql.
// These two statements retire the pre-shared-index shape on databases that
// carried it (local development only; the index was never in production).
const COMPAT: &str = "DROP TABLE IF EXISTS knowledge_sync_state;
ALTER TABLE knowledge_document_grants
    ADD COLUMN IF NOT EXISTS granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW();";

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
    sqlx::raw_sql(COMPAT).execute(pool).await?;
    sqlx::raw_sql(include_str!("../schema/google_drive.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn require(pool: &PgPool, database: &str) -> Result<(), KnowledgeBankError> {
    if !pgvector_available(pool).await? {
        return Err(KnowledgeBankError::NotConfigured(pgvector_missing(
            database,
        )));
    }
    apply(pool).await
}

#[must_use]
pub fn pgvector_missing(database: &str) -> String {
    format!(
        "knowledge-bank: database {database} has no pgvector; it is required (see \
         docs/install/knowledge-db.md)"
    )
}

// Why: `host:port/db` of a connection URL with the credentials dropped — the
// only form a boot-failure message may safely name.
#[must_use]
pub fn describe_database_url(url: &str) -> String {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host_and_path = rest.rsplit_once('@').map_or(rest, |(_, host)| host);
    host_and_path
        .split_once('?')
        .map_or(host_and_path, |(before, _)| before)
        .to_owned()
}

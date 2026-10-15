//! The clients declared in agent-farms.yaml. A client's id is its slug, so
//! a seed upserts by slug and never deletes: runs keep the client they were
//! attributed to.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::types::ClientId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct ClientRow {
    pub id: ClientId,
    pub name: String,
    pub slug: String,
    pub odoo_partner_id: Option<i64>,
    pub created_at: DateTime<Utc>,
}

pub async fn upsert_client(
    pool: &PgPool,
    slug: &str,
    name: &str,
    odoo_partner_id: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/clients/upsert_client.sql",
        slug,
        name,
        odoo_partner_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: lint-ok: unused-pub — the per-client farm page (WP3) lists clients.
pub async fn list_clients(pool: &PgPool) -> Result<Vec<ClientRow>, sqlx::Error> {
    sqlx::query_file_as!(ClientRow, "src/repositories/clients/list_clients.sql")
        .fetch_all(pool)
        .await
}

pub async fn find_client(pool: &PgPool, slug: &str) -> Result<Option<ClientRow>, sqlx::Error> {
    sqlx::query_file_as!(ClientRow, "src/repositories/clients/find_client.sql", slug)
        .fetch_optional(pool)
        .await
}

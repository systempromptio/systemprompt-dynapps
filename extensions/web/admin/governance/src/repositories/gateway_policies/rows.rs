//! `ai_gateway_policies` as the console reads and writes it.
//!
//! Core owns the table and its repository, but that repository is built on a
//! `DbPool` the admin handlers do not hold; these are the same statements
//! over the request's `PgPool`. `effective_spec` reproduces core's merge
//! (`PolicyResolver::merge`) so the quota page shows the windows the
//! gateway is actually enforcing, not one row's view of them: each non-empty
//! section of a later enabled row replaces the one before it outright, so
//! windows are never unioned across rows.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::ai::GatewayPolicySpec;
use systemprompt::identifiers::AiGatewayPolicyId;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Debug, Clone, Serialize)]
pub struct PolicyRow {
    pub name: String,
    pub spec: GatewayPolicySpec,
    pub enabled: bool,
    pub priority: i32,
    pub updated_at: DateTime<Utc>,
}

// Why: core's policy resolver skips a row whose spec does not parse; erroring
// here instead would take the whole page down for one bad row.
// JSON: JSONB column — the spec as core stored it, parsed into its type here
fn parse_row(
    name: String,
    spec: serde_json::Value,
    enabled: bool,
    priority: i32,
    updated_at: DateTime<Utc>,
) -> Option<PolicyRow> {
    match serde_json::from_value::<GatewayPolicySpec>(spec) {
        Ok(spec) => Some(PolicyRow {
            name,
            spec,
            enabled,
            priority,
            updated_at,
        }),
        Err(e) => {
            tracing::warn!(policy = %name, error = %e, "gateway policy spec malformed — skipped");
            None
        },
    }
}

pub async fn list_policies(pool: &PgPool) -> Result<Vec<PolicyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r"SELECT name, spec, enabled, priority, updated_at
            FROM ai_gateway_policies
           ORDER BY priority ASC, name ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| parse_row(r.name, r.spec, r.enabled, r.priority, r.updated_at))
        .collect())
}

pub async fn count_policies(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM ai_gateway_policies"#)
        .fetch_one(pool)
        .await
}

/// One policy to write: the same upsert core's loader performs, keyed on
/// the name, so an edit and a sync apply land on the same row.
#[derive(Debug, Clone)]
pub struct PolicyWrite<'a> {
    pub name: &'a str,
    pub spec: &'a GatewayPolicySpec,
    pub enabled: bool,
    pub priority: i32,
}

pub async fn upsert_policy(pool: &PgPool, write: &PolicyWrite<'_>) -> AdminResult<()> {
    let spec = serde_json::to_value(write.spec).map_err(AdminError::internal)?;
    let id = AiGatewayPolicyId::generate();
    sqlx::query!(
        r"INSERT INTO ai_gateway_policies (id, name, spec, enabled, priority, created_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
          ON CONFLICT (name) DO UPDATE
             SET spec = EXCLUDED.spec,
                 enabled = EXCLUDED.enabled,
                 priority = EXCLUDED.priority,
                 updated_at = CURRENT_TIMESTAMP",
        id.as_str(),
        write.name,
        spec,
        write.enabled,
        write.priority
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_policy(pool: &PgPool, name: &str) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!("DELETE FROM ai_gateway_policies WHERE name = $1", name)
        .execute(pool)
        .await?;
    Ok(done.rows_affected())
}

#[must_use]
pub fn effective_spec(rows: &[PolicyRow]) -> GatewayPolicySpec {
    let mut merged = GatewayPolicySpec::permissive();
    for row in rows.iter().filter(|r| r.enabled) {
        let spec = &row.spec;
        if !spec.quota_windows.is_empty() || spec.quota_mode.is_warn() {
            merged.quota_mode = spec.quota_mode;
        }
        if !spec.quota_windows.is_empty() {
            merged.quota_windows.clone_from(&spec.quota_windows);
        }
        if !spec.safety.scanners.is_empty()
            || !spec.safety.block_categories.is_empty()
            || !spec.safety.block_response_categories.is_empty()
            || spec.safety.mode.is_warn()
        {
            merged.safety = spec.safety.clone();
        }
    }
    merged
}

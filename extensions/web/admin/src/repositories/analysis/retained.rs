//! Skill drill-down figures read from retained reporting data only.
//!
//! Totals and per-resource rollups come from `analytics_snapshot_contributions`
//! (scope `''` is the organisation, scope `<resource id>` is one managed
//! resource); per-resource conversations walk `analytics_normalized_facts`
//! from an attributed invocation through its resource associations to the
//! requests and assessments they name. Nothing is recomputed from raw events,
//! so a resource appears only once the snapshot pipeline has drained its
//! facts, and an invocation that is not attributed to a managed resource is
//! excluded from every per-resource figure.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use systemprompt::identifiers::{ManagedResourceId, NativeSessionId, UserId};

pub const PAGE_SIZE: u32 = 50;

/// Half-open UTC window `[start, end)` applied to a fact's `occurred_at`, the
/// same timestamp the daily snapshot buckets it under.
#[derive(Debug, Clone, Copy)]
pub struct RetainedWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Copy, FromRow, Serialize)]
pub struct RetainedTotals {
    pub invocations: i64,
    pub attributed: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedResourceRow {
    pub resource_id: ManagedResourceId,
    pub resource_key: String,
    pub kind: String,
    pub invocations: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub total_rows: i64,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedConversationRow {
    pub session_id: NativeSessionId,
    pub user_id: UserId,
    pub host: Option<String>,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
    pub invocations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub assessment_status: Option<String>,
    pub score_millionths: Option<i64>,
    pub total_rows: i64,
}

fn offset_of(page: u32) -> i64 {
    i64::from(page.saturating_sub(1)) * i64::from(PAGE_SIZE)
}

pub async fn get_retained_totals(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
) -> Result<RetainedTotals, sqlx::Error> {
    sqlx::query_as::<_, RetainedTotals>(include_str!("retained_totals.sql"))
        .bind(owner.as_str())
        .bind(window.start)
        .bind(window.end)
        .fetch_one(pool)
        .await
}

pub async fn list_retained_resources(
    pool: &PgPool,
    owner: &UserId,
    window: RetainedWindow,
    page: u32,
) -> Result<(Vec<RetainedResourceRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, RetainedResourceRow>(include_str!("retained_resources.sql"))
        .bind(owner.as_str())
        .bind(window.start)
        .bind(window.end)
        .bind(i64::from(PAGE_SIZE))
        .bind(offset_of(page))
        .fetch_all(pool)
        .await?;
    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((rows, total))
}

pub async fn list_retained_conversations(
    pool: &PgPool,
    owner: &UserId,
    resource: &ManagedResourceId,
    window: RetainedWindow,
    page: u32,
) -> Result<(Vec<RetainedConversationRow>, i64), sqlx::Error> {
    let rows =
        sqlx::query_as::<_, RetainedConversationRow>(include_str!("retained_conversations.sql"))
            .bind(owner.as_str())
            .bind(resource.as_str())
            .bind(window.start)
            .bind(window.end)
            .bind(i64::from(PAGE_SIZE))
            .bind(offset_of(page))
            .fetch_all(pool)
            .await?;
    let total = rows.first().map_or(0, |row| row.total_rows);
    Ok((rows, total))
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RetainedResourceRef {
    pub resource_id: ManagedResourceId,
    pub resource_key: String,
    pub kind: String,
}

pub async fn find_retained_resource(
    pool: &PgPool,
    owner: &UserId,
    resource: &ManagedResourceId,
) -> Result<Option<RetainedResourceRef>, sqlx::Error> {
    sqlx::query_as::<_, RetainedResourceRef>(include_str!("retained_resource.sql"))
        .bind(owner.as_str())
        .bind(resource.as_str())
        .fetch_optional(pool)
        .await
}

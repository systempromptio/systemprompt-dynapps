//! Headline figures for the dashboard's KPI strip.
//!
//! The token total here must match the profile pane's exactly. The canonical
//! definition, and why it is what it is, lives in
//! [`crate::repositories::users::usage`]; the two are pinned together by
//! `tests/integration/admin-core/src/usage_reconciliation.rs`.
//!
//! Every `prev_*` figure covers the preceding window of equal width. Weekly
//! active users ignore the picked window: both WAU figures are trailing weeks
//! anchored to the statement's `NOW()`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Default, Clone, Copy)]
pub struct SiteKpis {
    pub total_requests: i64,
    pub error_count: i64,
    pub total_cost_microdollars: i64,
    pub total_tokens: i64,
    // Why: providers bill reasoning inside `output_tokens` (and so inside
    // `tokens_used`); report it as a share, never add it.
    pub reasoning_tokens: i64,
    pub output_tokens: i64,
    pub active_users: i64,
    pub weekly_active_users: i64,
    pub prev_total_requests: i64,
    pub prev_error_count: i64,
    pub prev_total_cost_microdollars: i64,
    pub prev_total_tokens: i64,
    pub prev_active_users: i64,
    pub prev_weekly_active_users: i64,
}

// Why: Postgres gives one statement one snapshot; both windows are counted in
// a single `FILTER` pass so a write landing mid-read cannot skew the delta.
pub async fn get_site_kpis(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SiteKpis, sqlx::Error> {
    let prev_from = equal_width_previous_window_start(&range);
    let row = sqlx::query_file!(
        "src/repositories/analytics/site/kpis.sql",
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        prev_from,
    )
    .fetch_one(pool)
    .await?;

    Ok(SiteKpis {
        total_requests: row.total,
        error_count: row.errors,
        total_cost_microdollars: row.cost,
        total_tokens: row.tokens,
        reasoning_tokens: row.reasoning_tokens,
        output_tokens: row.output_tokens,
        active_users: row.active_users,
        weekly_active_users: row.weekly_active_users,
        prev_total_requests: row.prev_total,
        prev_error_count: row.prev_errors,
        prev_total_cost_microdollars: row.prev_cost,
        prev_total_tokens: row.prev_tokens,
        prev_active_users: row.prev_active_users,
        prev_weekly_active_users: row.prev_weekly_active_users,
    })
}

fn equal_width_previous_window_start(range: &TimeRange) -> DateTime<Utc> {
    range.from - (range.to - range.from)
}

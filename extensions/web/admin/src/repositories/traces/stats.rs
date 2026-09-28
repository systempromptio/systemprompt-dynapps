//! Per-session percentile stats over the same window as the trace list.
//!
//! Every source of a session (tool events, governance decisions, requests)
//! carries a user id, so the one `subject_ids` predicate scopes all three.

use sqlx::PgPool;

use super::TraceStats;
use systemprompt_web_admin_core::util::time_range::TimeRange;

pub async fn get_trace_stats(
    pool: &PgPool,
    range: TimeRange,
    subject_ids: Option<&[String]>,
    include_lifecycle: bool,
) -> Result<TraceStats, sqlx::Error> {
    let row = sqlx::query_file!(
        "src/repositories/traces/stats.sql",
        range.from,
        range.to,
        subject_ids,
        include_lifecycle,
    )
    .fetch_one(pool)
    .await?;

    Ok(TraceStats {
        total_traces: row.total_traces,
        error_count: row.error_count,
        deny_count: row.deny_count,
        active_traces: row.active_traces,
        lifecycle_count: row.lifecycle_count,
        p50_active_ms: row.p50,
        p95_active_ms: row.p95,
        p99_active_ms: row.p99,
        total_cost_microdollars: row.total_cost,
        total_tokens: row.total_tokens,
    })
}

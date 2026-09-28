//! The time series behind the overview's charts and tile sparklines.
//!
//! A fixed bucket count rather than a fixed bucket width: every chart on the
//! page is the same 24-point shape whichever window is picked, so a chart's
//! height never depends on how many hours the reader asked for. At the
//! 24-hour window a bucket is exactly one hour, which is what the caption
//! says. One statement carries every per-bucket figure — requests, people,
//! conversations, spend, tokens, cache, errors, denials, latency, tool calls
//! — so a spike in one series lines up with the others to the bucket.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

pub const BUCKETS: i64 = 24;

/// One bucket of the overview's spine, every figure the charts plot.
#[derive(Debug, Clone, Copy, Default)]
pub struct OverviewSeriesPoint {
    pub requests: i64,
    pub users: i64,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub tokens: i64,
    pub cache_tokens: i64,
    pub errors: i64,
    pub rejected: i64,
    pub p50_latency_ms: i64,
    pub p95_latency_ms: i64,
    pub tool_calls: i64,
    pub denied: i64,
}

pub(crate) fn bucket_width_secs(range: TimeRange) -> f64 {
    let span_secs = (range.to - range.from).num_seconds().max(BUCKETS);
    ((span_secs as f64) / (BUCKETS as f64)).max(1.0)
}

pub(crate) fn bucket_count() -> i32 {
    i32::try_from(BUCKETS).unwrap_or(24)
}

pub async fn list_overview_series(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<OverviewSeriesPoint>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/overview/series.sql",
        range.from,
        range.to,
        bucket_count(),
        bucket_width_secs(range),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| OverviewSeriesPoint {
            requests: row.requests,
            users: row.users,
            conversations: row.conversations,
            cost_microdollars: row.cost,
            tokens: row.tokens,
            cache_tokens: row.cache_tokens,
            errors: row.errors,
            rejected: row.rejected,
            p50_latency_ms: row.p50,
            p95_latency_ms: row.p95,
            tool_calls: row.tool_calls,
            denied: row.denied,
        })
        .collect())
}

//! `gateway_overhead_daily`: measured gateway overhead per UTC day (NFR-1.6).
//!
//! Overhead is what the gateway adds around the provider call:
//! `latency_ms - upstream_latency_ms`, clamped at zero, over completed,
//! non-synthetic requests that carry both timings. The nightly
//! `gateway_overhead_rollup` job writes one row per wire dialect plus an
//! `all` row for a day in a single `GROUPING SETS` pass; the percentiles are
//! Postgres `percentile_cont` (linear interpolation), in milliseconds.
//! Re-running a day overwrites it, so a late or repeated run never
//! double-counts.

use chrono::NaiveDate;
use sqlx::PgPool;

pub const ALL_DIALECTS: &str = "all";

pub const KEEP_DAYS: i32 = 400;

pub const WARN_P95_MS: f64 = 250.0;

pub const ERR_P95_MS: f64 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GatewayOverheadDay {
    pub date: NaiveDate,
    pub request_count: i64,
    pub overhead_p50_ms: f64,
    pub overhead_p95_ms: f64,
    pub overhead_p99_ms: f64,
    pub upstream_p50_ms: f64,
    pub upstream_p95_ms: f64,
    pub upstream_p99_ms: f64,
}

pub fn overhead_tone(p95_ms: f64) -> &'static str {
    if p95_ms >= ERR_P95_MS {
        "err"
    } else if p95_ms >= WARN_P95_MS {
        "warn"
    } else {
        "ok"
    }
}

pub async fn upsert_gateway_overhead_for_day(
    pool: &PgPool,
    day: NaiveDate,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO gateway_overhead_daily
            (date, wire_protocol, request_count,
             overhead_p50_ms, overhead_p95_ms, overhead_p99_ms,
             upstream_p50_ms, upstream_p95_ms, upstream_p99_ms)
        SELECT
            $1::DATE,
            CASE WHEN GROUPING(wire_protocol) = 1 THEN $2::TEXT ELSE wire_protocol END,
            COUNT(*),
            percentile_cont(0.50) WITHIN GROUP (ORDER BY overhead_ms),
            percentile_cont(0.95) WITHIN GROUP (ORDER BY overhead_ms),
            percentile_cont(0.99) WITHIN GROUP (ORDER BY overhead_ms),
            percentile_cont(0.50) WITHIN GROUP (ORDER BY upstream_ms),
            percentile_cont(0.95) WITHIN GROUP (ORDER BY upstream_ms),
            percentile_cont(0.99) WITHIN GROUP (ORDER BY upstream_ms)
        FROM (
            SELECT wire_protocol,
                   GREATEST(latency_ms - upstream_latency_ms, 0)::FLOAT8 AS overhead_ms,
                   upstream_latency_ms::FLOAT8 AS upstream_ms
            FROM ai_requests
            WHERE status = 'completed'
              AND NOT synthetic
              AND latency_ms IS NOT NULL
              AND upstream_latency_ms IS NOT NULL
              AND created_at >= ($1::DATE)::TIMESTAMP AT TIME ZONE 'UTC'
              AND created_at < ($1::DATE + 1)::TIMESTAMP AT TIME ZONE 'UTC'
        ) AS timed
        GROUP BY GROUPING SETS ((), (wire_protocol))
        HAVING COUNT(*) > 0
        ON CONFLICT (date, wire_protocol) DO UPDATE SET
            request_count = EXCLUDED.request_count,
            overhead_p50_ms = EXCLUDED.overhead_p50_ms,
            overhead_p95_ms = EXCLUDED.overhead_p95_ms,
            overhead_p99_ms = EXCLUDED.overhead_p99_ms,
            upstream_p50_ms = EXCLUDED.upstream_p50_ms,
            upstream_p95_ms = EXCLUDED.upstream_p95_ms,
            upstream_p99_ms = EXCLUDED.upstream_p99_ms,
            computed_at = NOW()
        "#,
        day,
        ALL_DIALECTS,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

pub async fn delete_gateway_overhead_before(
    pool: &PgPool,
    cutoff: NaiveDate,
) -> Result<u64, sqlx::Error> {
    Ok(
        sqlx::query!("DELETE FROM gateway_overhead_daily WHERE date < $1", cutoff)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

pub async fn list_gateway_overhead_days(
    pool: &PgPool,
    days: i64,
) -> Result<Vec<GatewayOverheadDay>, sqlx::Error> {
    sqlx::query_as!(
        GatewayOverheadDay,
        r#"
        SELECT date, request_count,
               overhead_p50_ms, overhead_p95_ms, overhead_p99_ms,
               upstream_p50_ms, upstream_p95_ms, upstream_p99_ms
        FROM gateway_overhead_daily
        WHERE wire_protocol = $1
        ORDER BY date DESC
        LIMIT $2
        "#,
        ALL_DIALECTS,
        days,
    )
    .fetch_all(pool)
    .await
}

//! Latency-bucketed request split: the honest stand-in for "fast/slow
//! request pools", which this platform does not have.
//!
//! The threshold defaults to 5000 ms — one of the request page's histogram
//! bin edges, so the default split and that histogram can never disagree —
//! and is caller-configurable (`?slo_ms=`) so an SLO target other than 5s can
//! be reported against without a deploy. It is never derived from
//! percentiles, so period-over-period comparisons keep their meaning. Untimed
//! requests (NULL latency) are surfaced, not hidden. An out-of-range
//! `slo_ms` is clamped rather than rejected, so an edited URL still renders.

use sqlx::PgPool;

use systemprompt_web_admin_core::util::time_range::TimeRange;

use super::SiteScope;

pub const FAST_THRESHOLD_MS: i32 = 5_000;
const MIN_SLO_MS: i32 = 500;
const MAX_SLO_MS: i32 = 60_000;

pub fn resolve_slo_ms(requested: Option<i32>) -> i32 {
    requested.map_or(FAST_THRESHOLD_MS, |ms| ms.clamp(MIN_SLO_MS, MAX_SLO_MS))
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LatencySplit {
    pub fast: i64,
    pub slow: i64,
    pub untimed: i64,
    pub threshold_ms: i32,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

pub async fn get_latency_split(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    threshold_ms: i32,
) -> Result<LatencySplit, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE r.latency_ms < $5)::BIGINT AS "fast!",
            COUNT(*) FILTER (WHERE r.latency_ms >= $5)::BIGINT AS "slow!",
            COUNT(*) FILTER (WHERE r.latency_ms IS NULL)::BIGINT AS "untimed!",
            COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms), 0)
                AS "p50!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms), 0)
                AS "p95!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND r.traffic = 'production'
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        threshold_ms,
    )
    .fetch_one(pool)
    .await?;

    Ok(LatencySplit {
        fast: row.fast,
        slow: row.slow,
        untimed: row.untimed,
        threshold_ms,
        p50_ms: row.p50,
        p95_ms: row.p95,
    })
}

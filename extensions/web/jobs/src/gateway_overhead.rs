//! `gateway_overhead_rollup` job: nightly p50/p95/p99 of measured gateway
//! overhead (total minus upstream latency) for the previous UTC day, written
//! to `gateway_overhead_daily` and pruned to the last 400 days (NFR-1.6).
//!
//! Runs after midnight UTC so the day it rolls up is closed. Rolling up a day
//! overwrites that day's rows, so a manual re-run is safe.

use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin_analytics::repositories::overview::gateway_overhead::{
    KEEP_DAYS, delete_gateway_overhead_before, upsert_gateway_overhead_for_day,
};
use systemprompt_web_shared::error::MarketplaceError;

use crate::error::JobError;

pub fn previous_utc_day(today: NaiveDate) -> NaiveDate {
    today - Duration::days(1)
}

pub fn retention_cutoff(today: NaiveDate) -> NaiveDate {
    today - Duration::days(i64::from(KEEP_DAYS))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GatewayOverheadRollupJob;

impl GatewayOverheadRollupJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let today = Utc::now().date_naive();
        let day = previous_utc_day(today);
        let written = upsert_gateway_overhead_for_day(pool, day)
            .await
            .map_err(MarketplaceError::from)?;
        let pruned = delete_gateway_overhead_before(pool, retention_cutoff(today))
            .await
            .map_err(MarketplaceError::from)?;
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(%day, rows = written, pruned, duration_ms, "Gateway overhead rollup completed");
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for GatewayOverheadRollupJob {
    fn name(&self) -> &'static str {
        "gateway_overhead_rollup"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Rolls the previous UTC day's gateway overhead (total minus upstream latency) into p50/p95/p99 per wire dialect"
    }

    fn schedule(&self) -> &'static str {
        "0 10 0 * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Gateway overhead rollup invoked");

        let db = ctx.get::<DbPool>()?;
        let pool = db.write_pool();

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&GatewayOverheadRollupJob);

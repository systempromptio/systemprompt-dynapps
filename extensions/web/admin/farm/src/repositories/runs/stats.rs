//! Run aggregates: the stat strip over a filter and the daily sparkline.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::filters::RunFilter;
use crate::repositories::summaries::{DailyRunCount, RunStats};
use crate::types::{FarmAgentId, FarmId, RunStatus, TriggeredVia};

pub async fn get_run_stats(pool: &PgPool, filter: &RunFilter) -> Result<RunStats, sqlx::Error> {
    sqlx::query_file_as!(
        RunStats,
        "src/repositories/runs/get_run_stats.sql",
        filter.farm_id.as_ref().map(FarmId::as_str),
        filter.agent_id.as_ref().map(FarmAgentId::as_str),
        filter.user_id.as_ref().map(UserId::as_str),
        filter.status.map(RunStatus::as_str),
        filter.triggered_via.map(TriggeredVia::as_str),
        filter.since,
        filter.until,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_daily_run_counts(
    pool: &PgPool,
    since: DateTime<Utc>,
) -> Result<Vec<DailyRunCount>, sqlx::Error> {
    sqlx::query_file_as!(
        DailyRunCount,
        "src/repositories/runs/list_daily_run_counts.sql",
        since
    )
    .fetch_all(pool)
    .await
}

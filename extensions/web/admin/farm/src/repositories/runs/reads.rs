//! Run reads: one run, the paged and filtered list, and the queue views the
//! runner and the dashboard poll.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use crate::repositories::filters::{PageWindow, RunFilter, RunSort};
use crate::repositories::rows::RunRow;
use crate::types::{FarmAgentId, FarmId, FarmRunId, RunStatus, TriggeredVia};

pub async fn find_run(pool: &PgPool, id: &FarmRunId) -> Result<Option<RunRow>, sqlx::Error> {
    sqlx::query_file_as!(RunRow, "src/repositories/runs/find_run.sql", id.as_str())
        .fetch_optional(pool)
        .await
}

pub async fn list_runs_paged(
    pool: &PgPool,
    filter: &RunFilter,
    sort: RunSort,
    page: PageWindow,
) -> Result<(Vec<RunRow>, i64), sqlx::Error> {
    let farm_id = filter.farm_id.as_ref().map(FarmId::as_str);
    let agent_id = filter.agent_id.as_ref().map(FarmAgentId::as_str);
    let user_id = filter.user_id.as_ref().map(UserId::as_str);
    let status = filter.status.map(RunStatus::as_str);
    let via = filter.triggered_via.map(TriggeredVia::as_str);
    let rows = sqlx::query_file_as!(
        RunRow,
        "src/repositories/runs/list_runs_paged.sql",
        farm_id,
        agent_id,
        user_id,
        status,
        via,
        filter.since,
        filter.until,
        sort.sql_key(),
        page.limit,
        page.offset,
    )
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_file_scalar!(
        "src/repositories/runs/count_runs.sql",
        farm_id,
        agent_id,
        user_id,
        status,
        via,
        filter.since,
        filter.until,
    )
    .fetch_one(pool)
    .await?;
    Ok((rows, total))
}

pub async fn list_active_runs(pool: &PgPool) -> Result<Vec<RunRow>, sqlx::Error> {
    sqlx::query_file_as!(RunRow, "src/repositories/runs/list_active_runs.sql")
        .fetch_all(pool)
        .await
}

pub async fn list_recent_runs(
    pool: &PgPool,
    farm_id: Option<&FarmId>,
    agent_id: Option<&FarmAgentId>,
    limit: i64,
) -> Result<Vec<RunRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunRow,
        "src/repositories/runs/list_recent_runs.sql",
        farm_id.map(FarmId::as_str),
        agent_id.map(FarmAgentId::as_str),
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_runs_to_requeue(pool: &PgPool) -> Result<Vec<RunRow>, sqlx::Error> {
    sqlx::query_file_as!(RunRow, "src/repositories/runs/list_runs_to_requeue.sql")
        .fetch_all(pool)
        .await
}

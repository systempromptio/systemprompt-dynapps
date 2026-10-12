//! Run lifecycle writes. Each transition is guarded by the statuses it may
//! leave, so a run that was cancelled or finished cannot be moved back.

use sqlx::{PgExecutor, PgPool};
use systemprompt::identifiers::{AgentRunId, ContextId, UserId};

use crate::repositories::filters::{NewRun, RunFinish};
use crate::repositories::rows::RunRow;
use crate::types::{EvalResultId, FarmAgentId, FarmId, FarmRunId};

pub async fn insert_run<'e>(
    executor: impl PgExecutor<'e>,
    run: &NewRun,
) -> Result<RunRow, sqlx::Error> {
    sqlx::query_file_as!(
        RunRow,
        "src/repositories/runs/insert_run.sql",
        run.id.as_str(),
        run.agent_run_id.as_str(),
        run.farm_id.as_str(),
        run.agent_id.as_str(),
        run.user_id.as_str(),
        run.triggered_via.as_str(),
        run.host,
        run.attempt,
        run.prompt,
        run.fixture,
        run.model,
        run.permission_mode,
        run.max_turns,
        run.max_budget_usd,
        run.timeout_seconds,
        run.source_commit_sha,
        run.eval_result_id.as_ref().map(EvalResultId::as_str),
    )
    .fetch_one(executor)
    .await
}

pub async fn update_run_started(pool: &PgPool, id: &FarmRunId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!("src/repositories/runs/update_run_started.sql", id.as_str())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_run_finished(
    pool: &PgPool,
    id: &FarmRunId,
    finish: &RunFinish,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_finished.sql",
        id.as_str(),
        finish.status.as_str(),
        finish.error_class,
        finish.cost_microdollars,
        finish.event_count,
        finish.tool_call_count,
        finish.gateway_request_count,
        finish.context_id.as_ref().map(ContextId::as_str),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_run_cancel_requested(
    pool: &PgPool,
    id: &FarmRunId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_cancel_requested.sql",
        id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

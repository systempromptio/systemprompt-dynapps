//! Run lifecycle writes. Each transition is guarded by the statuses it may
//! leave, so a run that was cancelled or finished cannot be moved back.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use crate::repositories::filters::{NewRun, RunOutcome};
use crate::repositories::rows::RunRow;
use crate::types::{EvalResultId, FarmAgentId, FarmId, FarmRunId};

pub async fn insert_run(pool: &PgPool, run: &NewRun) -> Result<RunRow, sqlx::Error> {
    sqlx::query_file_as!(
        RunRow,
        "src/repositories/runs/insert_run.sql",
        run.id.as_str(),
        run.farm_id.as_str(),
        run.agent_id.as_str(),
        run.user_id.as_str(),
        run.triggered_via.as_str(),
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
    .fetch_one(pool)
    .await
}

pub async fn update_run_preparing(pool: &PgPool, id: &FarmRunId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_preparing.sql",
        id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_run_started(
    pool: &PgPool,
    id: &FarmRunId,
    gateway_session_id: &str,
    api_key_id: &str,
    run_dir: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_started.sql",
        id.as_str(),
        gateway_session_id,
        api_key_id,
        run_dir,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_run_pid(
    pool: &PgPool,
    id: &FarmRunId,
    pid: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!("src/repositories/runs/update_run_pid.sql", id.as_str(), pid)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_run_claude_session(
    pool: &PgPool,
    id: &FarmRunId,
    claude_session_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/runs/update_run_claude_session.sql",
        id.as_str(),
        claude_session_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_run_finished(
    pool: &PgPool,
    id: &FarmRunId,
    outcome: &RunOutcome,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/runs/update_run_finished.sql",
        id.as_str(),
        outcome.status.as_str(),
        outcome.context_id.as_ref().map(ContextId::as_str),
        outcome.duration_ms,
        outcome.num_turns,
        outcome.input_tokens,
        outcome.output_tokens,
        outcome.cost_microdollars,
        outcome.result_subtype,
        outcome.exit_code,
        outcome.error,
        outcome.diff_summary,
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

pub async fn update_orphaned_runs_failed(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query_file!("src/repositories/runs/update_orphaned_runs_failed.sql")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

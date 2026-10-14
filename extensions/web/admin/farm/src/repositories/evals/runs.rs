//! Eval runs: one per launch of a suite, with the tally the orchestrator
//! keeps written back as each case finishes.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::rows::EvalRunRow;
use crate::types::{EvalRunId, EvalRunStatus, EvalSuiteId, FarmAgentId, FarmId, TriggeredVia};

/// The identity of an eval run about to start.
#[derive(Debug, Clone, Copy)]
pub struct NewEvalRun<'a> {
    pub id: &'a EvalRunId,
    pub suite_id: &'a EvalSuiteId,
    pub farm_id: &'a FarmId,
    pub agent_id: &'a FarmAgentId,
    pub user_id: &'a UserId,
    pub triggered_via: TriggeredVia,
}

pub async fn insert_eval_run(
    pool: &PgPool,
    run: &NewEvalRun<'_>,
) -> Result<EvalRunRow, sqlx::Error> {
    sqlx::query_file_as!(
        EvalRunRow,
        "src/repositories/evals/insert_eval_run.sql",
        run.id.as_str(),
        run.suite_id.as_str(),
        run.farm_id.as_str(),
        run.agent_id.as_str(),
        run.user_id.as_str(),
        run.triggered_via.as_str(),
    )
    .fetch_one(pool)
    .await
}

pub async fn find_eval_run(
    pool: &PgPool,
    id: &EvalRunId,
) -> Result<Option<EvalRunRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalRunRow,
        "src/repositories/evals/find_eval_run.sql",
        id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_eval_runs_paged(
    pool: &PgPool,
    farm_id: Option<&FarmId>,
    suite_id: Option<&EvalSuiteId>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<EvalRunRow>, i64), sqlx::Error> {
    let farm = farm_id.map(FarmId::as_str);
    let suite = suite_id.map(EvalSuiteId::as_str);
    let rows = sqlx::query_file_as!(
        EvalRunRow,
        "src/repositories/evals/list_eval_runs_paged.sql",
        farm,
        suite,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_file_scalar!("src/repositories/evals/count_eval_runs.sql", farm, suite)
        .fetch_one(pool)
        .await?;
    Ok((rows, total))
}

pub async fn list_recent_eval_runs(
    pool: &PgPool,
    farm_id: Option<&FarmId>,
    suite_id: Option<&EvalSuiteId>,
    limit: i64,
) -> Result<Vec<EvalRunRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalRunRow,
        "src/repositories/evals/list_recent_eval_runs.sql",
        farm_id.map(FarmId::as_str),
        suite_id.map(EvalSuiteId::as_str),
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_eval_run_status(
    pool: &PgPool,
    id: &EvalRunId,
    status: EvalRunStatus,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/update_eval_run_status.sql",
        id.as_str(),
        status.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_eval_run_counts(
    pool: &PgPool,
    id: &EvalRunId,
    passed: i32,
    failed: i32,
    errored: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/update_eval_run_counts.sql",
        id.as_str(),
        passed,
        failed,
        errored,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_eval_run_finished(
    pool: &PgPool,
    id: &EvalRunId,
    status: EvalRunStatus,
    mean_score: Option<f64>,
    cost_microdollars: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/update_eval_run_finished.sql",
        id.as_str(),
        status.as_str(),
        mean_score,
        cost_microdollars,
    )
    .execute(pool)
    .await?;
    Ok(())
}

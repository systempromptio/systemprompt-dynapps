//! Per-case eval results: created pending, linked to the farm run that
//! executes the case, then given the outcome the assertions and judge decide.

use sqlx::PgPool;

use crate::repositories::rows::EvalResultRow;
use crate::types::{EvalResultId, EvalResultStatus, EvalRunId, FarmRunId};

pub async fn insert_eval_result(
    pool: &PgPool,
    id: &EvalResultId,
    eval_run_id: &EvalRunId,
    case_id: &str,
) -> Result<EvalResultRow, sqlx::Error> {
    sqlx::query_file_as!(
        EvalResultRow,
        "src/repositories/evals/insert_eval_result.sql",
        id.as_str(),
        eval_run_id.as_str(),
        case_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_eval_result(
    pool: &PgPool,
    id: &EvalResultId,
) -> Result<Option<EvalResultRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalResultRow,
        "src/repositories/evals/find_eval_result.sql",
        id.as_str()
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_eval_results(
    pool: &PgPool,
    eval_run_id: &EvalRunId,
) -> Result<Vec<EvalResultRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalResultRow,
        "src/repositories/evals/list_eval_results.sql",
        eval_run_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn update_eval_result_run(
    pool: &PgPool,
    id: &EvalResultId,
    run_id: &FarmRunId,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/update_eval_result_run.sql",
        id.as_str(),
        run_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// A case's settled outcome, written back onto its result row.
#[derive(Debug, Clone, Copy)]
pub struct EvalOutcomeUpdate<'a> {
    pub status: EvalResultStatus,
    // JSON: fixed-shape document — each assertion's outcome as the evals
    // service serialized it.
    pub assertions: &'a serde_json::Value,
    pub judge_score: Option<f64>,
    pub judge_rationale: Option<&'a str>,
}

pub async fn update_eval_result_outcome(
    pool: &PgPool,
    id: &EvalResultId,
    outcome: &EvalOutcomeUpdate<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query_file!(
        "src/repositories/evals/update_eval_result_outcome.sql",
        id.as_str(),
        outcome.status.as_str(),
        outcome.assertions,
        outcome.judge_score,
        outcome.judge_rationale,
    )
    .execute(pool)
    .await?;
    Ok(())
}

//! `agent_farm_run_reports`: one task-completion report per terminal run.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::types::{ClientId, EvalResultId, FarmRunId};

// JSON: fixed-shape documents — `outputs` is a list of {path, bytes, sha256},
// `odoo_writes` a list of {tool, model, record_ids, decision, policy, at}.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct RunReportRow {
    pub run_id: FarmRunId,
    pub client_id: Option<ClientId>,
    pub completed: bool,
    pub summary: Option<String>,
    pub final_message: Option<String>,
    pub outputs: serde_json::Value,
    pub odoo_writes: serde_json::Value,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub created_at: DateTime<Utc>,
}

/// What the observer read from the run's events and workspace.
#[derive(Debug, Clone)]
pub struct NewRunReport<'a> {
    pub run_id: &'a FarmRunId,
    pub summary: Option<&'a str>,
    pub final_message: Option<&'a str>,
    // JSON: the list of {path, bytes, sha256} the run's output path holds.
    pub outputs: &'a serde_json::Value,
}

pub async fn upsert_run_report(
    pool: &PgPool,
    report: &NewRunReport<'_>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/reports/upsert_run_report.sql",
        report.run_id.as_str(),
        report.summary,
        report.final_message,
        report.outputs,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_report_eval(
    pool: &PgPool,
    result_id: &EvalResultId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!(
        "src/repositories/reports/update_report_eval.sql",
        result_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn find_run_report(
    pool: &PgPool,
    run_id: &FarmRunId,
) -> Result<Option<RunReportRow>, sqlx::Error> {
    sqlx::query_file_as!(
        RunReportRow,
        "src/repositories/reports/find_run_report.sql",
        run_id.as_str()
    )
    .fetch_optional(pool)
    .await
}

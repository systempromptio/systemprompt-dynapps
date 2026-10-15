//! `agent_farm_run_reports`: one task-completion report per terminal run.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::types::{ClientId, EvalResultId, FarmRunId};

// JSON: fixed-shape documents — `outputs` is a list of {path, bytes, sha256},
// `mcp_calls` a list of {server, reads, writes, failed}.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct RunReportRow {
    pub run_id: FarmRunId,
    pub client_id: Option<ClientId>,
    pub completed: bool,
    pub summary: Option<String>,
    pub final_message: Option<String>,
    pub outputs: serde_json::Value,
    pub mcp_calls: serde_json::Value,
    pub mcp_reads: i32,
    pub mcp_writes: i32,
    pub governance_warnings: i32,
    pub delivery_status: String,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub triggered_via: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub cost_microdollars: i64,
    pub turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

/// The scheduler-started runs over a window: the nightly audit flow's strip
/// on `/admin/farm`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScheduledStatsRow {
    pub runs: i64,
    pub succeeded: i64,
    pub cost_microdollars: i64,
    pub mcp_reads: i64,
    pub mcp_writes: i64,
    pub governance_warnings: i64,
    pub delivered: i64,
    pub reported: i64,
    pub eval_mean: Option<f64>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_run_id: Option<FarmRunId>,
    pub last_status: Option<String>,
}

/// What the observer read from the run's events and workspace.
#[derive(Debug, Clone)]
pub struct NewRunReport<'a> {
    pub run_id: &'a FarmRunId,
    pub summary: Option<&'a str>,
    pub final_message: Option<&'a str>,
    // JSON: the list of {path, bytes, sha256} the run's output path holds.
    pub outputs: &'a serde_json::Value,
    // JSON: the list of {server, reads, writes, failed} the run's MCP calls make.
    pub mcp_calls: &'a serde_json::Value,
    pub mcp_reads: i32,
    pub mcp_writes: i32,
    pub governance_warnings: i32,
    pub delivery_status: &'a str,
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
        report.mcp_calls,
        report.mcp_reads,
        report.mcp_writes,
        report.governance_warnings,
        report.delivery_status,
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

pub async fn list_runs_missing_report(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<FarmRunId>, sqlx::Error> {
    Ok(sqlx::query_file!(
        "src/repositories/reports/list_runs_missing_report.sql",
        limit
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| row.id)
    .collect())
}

pub async fn count_run_warnings(
    pool: &PgPool,
    session_ids: &[String],
    context: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_file_scalar!(
        "src/repositories/reports/count_run_warnings.sql",
        session_ids,
        context
    )
    .fetch_one(pool)
    .await
}

pub async fn get_scheduled_stats(
    pool: &PgPool,
    since: DateTime<Utc>,
) -> Result<ScheduledStatsRow, sqlx::Error> {
    sqlx::query_file_as!(
        ScheduledStatsRow,
        "src/repositories/reports/get_scheduled_stats.sql",
        since
    )
    .fetch_one(pool)
    .await
}

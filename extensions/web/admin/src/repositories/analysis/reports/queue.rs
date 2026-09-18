//! The `analysis_reports` queue: a request from the console, a fenced lease
//! for the job, completion with the model's findings, and retry. Every
//! statement is bound; the job never interpolates.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;

use super::reads::ReportRecord;
use super::{AnalysisReportRow, ReportFindings, ReportInputs};

/// One request as the console makes it.
#[derive(Debug, Clone)]
pub struct NewReport {
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub scope_label: Option<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub requested_by: String,
    pub inputs: ReportInputs,
}

/// What the job writes back after the one model call.
#[derive(Debug)]
pub struct ReportCompletion<'a> {
    pub id: &'a str,
    pub lease_token: &'a str,
    pub findings: &'a ReportFindings,
    pub provider: &'a str,
    pub model: &'a str,
    pub ai_request_id: Option<&'a str>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

pub async fn insert_report_request(
    pool: &PgPool,
    report: NewReport,
) -> Result<String, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO analysis_reports
               (id, scope_kind, scope_id, scope_label, window_start, window_end, requested_by, inputs)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        id,
        report.scope_kind,
        report.scope_id,
        report.scope_label,
        report.window_start,
        report.window_end,
        report.requested_by,
        Json(report.inputs) as _
    )
    .execute(pool)
    .await?;
    Ok(id)
}

// Why: a fenced lease — the returned rows carry the token the completion
// must present, so a run that outlived its lease cannot overwrite a newer one.
pub async fn claim_pending_reports(
    pool: &PgPool,
    limit: i64,
    lease_seconds: i64,
) -> Result<Vec<AnalysisReportRow>, sqlx::Error> {
    let token = uuid::Uuid::new_v4().to_string();
    let rows = sqlx::query_as!(
        ReportRecord,
        r#"WITH pending AS (
               SELECT id FROM analysis_reports
               WHERE status = 'pending' AND next_attempt <= clock_timestamp()
                 AND (lease_until IS NULL OR lease_until <= clock_timestamp())
               ORDER BY next_attempt, created_at LIMIT $1 FOR UPDATE SKIP LOCKED)
           UPDATE analysis_reports r
           SET lease_token = $2, lease_until = clock_timestamp() + $3::bigint * interval '1 second',
               attempts = r.attempts + 1, updated_at = clock_timestamp()
           FROM pending WHERE r.id = pending.id
           RETURNING r.id, r.scope_kind, r.scope_id, r.scope_label, r.window_start, r.window_end,
                     r.status, r.requested_by, r.provider, r.model, r.ai_request_id,
                     r.input_tokens, r.output_tokens, r.cost_microdollars,
                     r.inputs AS "inputs: Json<ReportInputs>",
                     r.findings AS "findings?: Json<ReportFindings>", r.attempts, r.lease_token,
                     r.last_error, r.generated_at, r.created_at, r.updated_at"#,
        limit,
        token,
        lease_seconds
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalysisReportRow::from).collect())
}

// Why: the call's cost is settled by the gateway after the response returns,
// so it is read from ai_requests here; a later run may re-read it.
pub async fn update_report_completion(
    pool: &PgPool,
    done: ReportCompletion<'_>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE analysis_reports
           SET status = 'generated', findings = $3, provider = $4, model = $5, ai_request_id = $6,
               input_tokens = $7, output_tokens = $8,
               cost_microdollars = (SELECT r.cost_microdollars FROM ai_requests r WHERE r.id = $6),
               generated_at = clock_timestamp(), lease_token = NULL, lease_until = NULL,
               last_error = NULL, updated_at = clock_timestamp()
           WHERE id = $1 AND lease_token = $2"#,
        done.id,
        done.lease_token,
        Json(done.findings) as _,
        done.provider,
        done.model,
        done.ai_request_id,
        done.input_tokens,
        done.output_tokens
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// Why: backs off on failure; the third failure parks the row as `failed` so
// a broken digest or provider does not spend budget forever.
pub async fn fail_report(
    pool: &PgPool,
    id: &str,
    lease_token: &str,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE analysis_reports
           SET status = CASE WHEN attempts >= 3 THEN 'failed' ELSE 'pending' END,
               lease_token = NULL, lease_until = NULL,
               next_attempt = clock_timestamp() + attempts * interval '2 minutes',
               last_error = LEFT($3, 500), updated_at = clock_timestamp()
           WHERE id = $1 AND lease_token = $2"#,
        id,
        lease_token,
        error
    )
    .execute(pool)
    .await?;
    Ok(())
}

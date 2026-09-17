//! Generated report rows as the Evaluation pages read them: the latest per
//! scope, one by id, the pending or failed row for a scope, and the history.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::ReportScope;

/// One report as the pages read it.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationReportRow {
    pub id: String,
    pub scope: ReportScope,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub marketplace_hash: Option<String>,
    pub status: String,
    pub trigger: String,
    pub requested_by: Option<UserId>,
    pub source_run_count: i64,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub ai_request_id: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub latency_ms: Option<i32>,
    pub inputs: serde_json::Value,
    pub findings: Option<serde_json::Value>,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

struct ReportRecord {
    id: String,
    scope_kind: String,
    scope_id: String,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    marketplace_hash: Option<String>,
    status: String,
    trigger: String,
    requested_by: Option<String>,
    source_run_count: i64,
    provider: Option<String>,
    model: Option<String>,
    ai_request_id: Option<String>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    cost_microdollars: Option<i64>,
    latency_ms: Option<i32>,
    inputs: serde_json::Value,
    findings: Option<serde_json::Value>,
    attempts: i32,
    last_error: Option<String>,
    generated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl ReportRecord {
    fn into_row(self) -> Option<EvaluationReportRow> {
        Some(EvaluationReportRow {
            scope: ReportScope::from_parts(&self.scope_kind, &self.scope_id)?,
            id: self.id,
            window_start: self.window_start,
            window_end: self.window_end,
            marketplace_hash: self.marketplace_hash,
            status: self.status,
            trigger: self.trigger,
            requested_by: self.requested_by.map(UserId::new),
            source_run_count: self.source_run_count,
            provider: self.provider,
            model: self.model,
            ai_request_id: self.ai_request_id,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cost_microdollars: self.cost_microdollars,
            latency_ms: self.latency_ms,
            inputs: self.inputs,
            findings: self.findings,
            attempts: self.attempts,
            last_error: self.last_error,
            generated_at: self.generated_at,
            created_at: self.created_at,
        })
    }
}

pub async fn find_latest_report(
    pool: &PgPool,
    scope: &ReportScope,
) -> Result<Option<EvaluationReportRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, window_start, window_end, marketplace_hash, status,
                  trigger, requested_by, source_run_count, provider, model, ai_request_id,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, inputs, findings,
                  attempts, last_error, generated_at, created_at
             FROM evaluation_reports
            WHERE scope_kind = $1 AND scope_id = $2 AND status = 'generated'
            ORDER BY generated_at DESC LIMIT 1"#,
        scope.kind(),
        scope.id(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(ReportRecord::into_row))
}

pub async fn find_report(
    pool: &PgPool,
    id: &str,
) -> Result<Option<EvaluationReportRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, window_start, window_end, marketplace_hash, status,
                  trigger, requested_by, source_run_count, provider, model, ai_request_id,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, inputs, findings,
                  attempts, last_error, generated_at, created_at
             FROM evaluation_reports WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(ReportRecord::into_row))
}

/// The pending or failed row for a scope, so the page can say a report is
/// on its way or why the last attempt failed.
pub async fn find_pending_report(
    pool: &PgPool,
    scope: &ReportScope,
) -> Result<Option<EvaluationReportRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, window_start, window_end, marketplace_hash, status,
                  trigger, requested_by, source_run_count, provider, model, ai_request_id,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, inputs, findings,
                  attempts, last_error, generated_at, created_at
             FROM evaluation_reports
            WHERE scope_kind = $1 AND scope_id = $2 AND status <> 'generated'
            ORDER BY created_at DESC LIMIT 1"#,
        scope.kind(),
        scope.id(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(ReportRecord::into_row))
}

pub async fn list_reports_paged(
    pool: &PgPool,
    scope: Option<&ReportScope>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<EvaluationReportRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id, scope_kind, scope_id, window_start, window_end, marketplace_hash, status,
                  trigger, requested_by, source_run_count, provider, model, ai_request_id,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, inputs, findings,
                  attempts, last_error, generated_at, created_at,
                  count(*) OVER () AS "total!"
             FROM evaluation_reports
            WHERE ($1::text IS NULL OR (scope_kind = $1 AND scope_id = $2))
            ORDER BY created_at DESC, id
            LIMIT $3 OFFSET $4"#,
        scope.map(ReportScope::kind),
        scope.map_or("", ReportScope::id),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;
    let total = rows.first().map_or(0, |r| r.total);
    let rows = rows
        .into_iter()
        .filter_map(|r| {
            ReportRecord {
                id: r.id,
                scope_kind: r.scope_kind,
                scope_id: r.scope_id,
                window_start: r.window_start,
                window_end: r.window_end,
                marketplace_hash: r.marketplace_hash,
                status: r.status,
                trigger: r.trigger,
                requested_by: r.requested_by,
                source_run_count: r.source_run_count,
                provider: r.provider,
                model: r.model,
                ai_request_id: r.ai_request_id,
                input_tokens: r.input_tokens,
                output_tokens: r.output_tokens,
                cost_microdollars: r.cost_microdollars,
                latency_ms: r.latency_ms,
                inputs: r.inputs,
                findings: r.findings,
                attempts: r.attempts,
                last_error: r.last_error,
                generated_at: r.generated_at,
                created_at: r.created_at,
            }
            .into_row()
        })
        .collect();
    Ok((rows, total))
}

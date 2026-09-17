//! The `evaluation_reports` table (schema 43): which scopes are due a fresh
//! report, the fenced leases the generator works under, and the generated
//! rows the Evaluation pages read.
//!
//! A scope is the whole instance (`global`) or one marketplace. Every
//! generation is its own row, so the latest report can be read beside the
//! one before it; at most one row per scope is pending at a time.

mod rows;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub use rows::{
    EvaluationReportRow, find_latest_report, find_pending_report, find_report, list_reports_paged,
};

/// Which slice of the judge's output a report reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportScope {
    Global,
    Marketplace(String),
}

impl ReportScope {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Marketplace(_) => "marketplace",
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        match self {
            Self::Global => "",
            Self::Marketplace(id) => id.as_str(),
        }
    }

    #[must_use]
    pub fn from_parts(kind: &str, id: &str) -> Option<Self> {
        match kind {
            "global" => Some(Self::Global),
            "marketplace" if !id.is_empty() => Some(Self::Marketplace(id.to_owned())),
            _ => None,
        }
    }
}

/// When a scope is re-read without anyone asking.
#[derive(Debug, Clone, Copy)]
pub struct Staleness {
    pub window_days: i32,
    pub min_new_runs: i64,
    pub max_age_hours: i32,
}

// Why: one statement decides every scope at once — the instance plus every
// marketplace with a recorded version — and the partial unique index turns
// a scope already pending into a no-op.
pub async fn insert_stale_scopes(pool: &PgPool, rule: Staleness) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"WITH scopes AS (
               SELECT 'global'::text AS scope_kind, ''::text AS scope_id
               UNION
               SELECT 'marketplace', marketplace_id FROM marketplace_versions
           ), latest AS (
               SELECT DISTINCT ON (scope_kind, scope_id) scope_kind, scope_id, generated_at, source_last_run_at
                 FROM evaluation_reports WHERE status = 'generated'
                ORDER BY scope_kind, scope_id, generated_at DESC
           ), fresh AS (
               SELECT s.scope_kind, s.scope_id,
                      (SELECT COUNT(*) FROM evaluation_runs r
                        WHERE r.status = 'succeeded'
                          AND r.created_at > COALESCE(l.source_last_run_at, '-infinity'::timestamptz)
                          AND (s.scope_kind = 'global' OR EXISTS (
                                SELECT 1 FROM evaluation_session_skills k
                                 WHERE k.session_id = r.session_id AND k.marketplace_id = s.scope_id))
                      ) AS new_runs,
                      l.generated_at
                 FROM scopes s LEFT JOIN latest l USING (scope_kind, scope_id)
           )
           INSERT INTO evaluation_reports (scope_kind, scope_id, window_start, window_end, marketplace_hash)
           SELECT f.scope_kind, f.scope_id,
                  clock_timestamp() - make_interval(days => $1), clock_timestamp(),
                  CASE WHEN f.scope_kind = 'marketplace'
                       THEN marketplace_version_at(f.scope_id, clock_timestamp()) END
             FROM fresh f
            WHERE f.new_runs > 0
              AND (f.generated_at IS NULL OR f.new_runs >= $2
                   OR f.generated_at < clock_timestamp() - make_interval(hours => $3))
           ON CONFLICT (scope_kind, scope_id) WHERE status = 'pending' DO NOTHING"#,
        rule.window_days,
        rule.min_new_runs,
        rule.max_age_hours,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

// Why: a click ignores the staleness rule; a pending row for the scope is
// re-stamped as manual and made due now instead of duplicated.
pub async fn insert_manual_report(
    pool: &PgPool,
    scope: &ReportScope,
    window_days: i32,
    requested_by: &UserId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO evaluation_reports (scope_kind, scope_id, window_start, window_end, marketplace_hash, trigger, requested_by)
           VALUES ($1, $2, clock_timestamp() - make_interval(days => $3), clock_timestamp(),
                   CASE WHEN $1 = 'marketplace' THEN marketplace_version_at($2, clock_timestamp()) END,
                   'manual', $4)
           ON CONFLICT (scope_kind, scope_id) WHERE status = 'pending' DO UPDATE
              SET trigger = 'manual', requested_by = EXCLUDED.requested_by,
                  window_start = EXCLUDED.window_start, window_end = EXCLUDED.window_end,
                  next_attempt = clock_timestamp(), attempts = 0, last_error = NULL,
                  lease_token = NULL, lease_until = NULL, updated_at = clock_timestamp()"#,
        scope.kind(),
        scope.id(),
        window_days,
        requested_by.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ReportLease {
    pub id: String,
    pub scope: ReportScope,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub marketplace_hash: Option<String>,
    pub attempts: i32,
    pub token: String,
}

pub async fn claim_report_leases(
    pool: &PgPool,
    limit: i64,
    scope: Option<&ReportScope>,
) -> Result<Vec<ReportLease>, sqlx::Error> {
    let token = uuid::Uuid::new_v4().to_string();
    let rows = sqlx::query!(
        r#"WITH due AS (
               SELECT id FROM evaluation_reports
                WHERE status = 'pending' AND next_attempt <= clock_timestamp()
                  AND (lease_until IS NULL OR lease_until <= clock_timestamp())
                  AND ($3::text IS NULL OR (scope_kind = $3 AND scope_id = $4))
                ORDER BY next_attempt, id LIMIT $1 FOR UPDATE SKIP LOCKED)
           UPDATE evaluation_reports r
              SET lease_token = $2, lease_until = clock_timestamp() + interval '10 minutes',
                  attempts = r.attempts + 1, updated_at = clock_timestamp()
             FROM due WHERE r.id = due.id
           RETURNING r.id, r.scope_kind, r.scope_id, r.window_start, r.window_end,
                     r.marketplace_hash, r.attempts"#,
        limit,
        token,
        scope.map(ReportScope::kind),
        scope.map_or("", ReportScope::id),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(ReportLease {
                scope: ReportScope::from_parts(&row.scope_kind, &row.scope_id)?,
                id: row.id,
                window_start: row.window_start,
                window_end: row.window_end,
                marketplace_hash: row.marketplace_hash,
                attempts: row.attempts,
                token: token.clone(),
            })
        })
        .collect())
}

// JSON: `inputs` is the digest the model was shown and `findings` its
// answer, both stored as the model saw and wrote them; the typed shapes live
// in services::evaluation::report::schema and are re-parsed on read.
/// Everything the generator learned, written onto the leased row.
#[derive(Debug)]
pub struct GeneratedReport<'a> {
    pub provider: &'a str,
    pub model: &'a str,
    pub ai_request_id: Option<&'a str>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub latency_ms: Option<i32>,
    pub inputs: &'a serde_json::Value,
    pub findings: &'a serde_json::Value,
    pub source_run_count: i64,
    pub source_last_run_at: Option<DateTime<Utc>>,
}

pub async fn update_report_generated(
    pool: &PgPool,
    lease: &ReportLease,
    done: GeneratedReport<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE evaluation_reports
              SET status = 'generated', provider = $3, model = $4, ai_request_id = $5,
                  input_tokens = $6, output_tokens = $7, latency_ms = $8, inputs = $9, findings = $10,
                  source_run_count = $11, source_last_run_at = $12,
                  generated_at = clock_timestamp(), lease_token = NULL, lease_until = NULL,
                  last_error = NULL, updated_at = clock_timestamp()
            WHERE id = $1 AND lease_token = $2"#,
        lease.id,
        lease.token,
        done.provider,
        done.model,
        done.ai_request_id,
        done.input_tokens,
        done.output_tokens,
        done.latency_ms,
        done.inputs,
        done.findings,
        done.source_run_count,
        done.source_last_run_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: backs off on failure; the fifth failure parks the row as `failed`
// until a click re-queues the scope.
pub async fn update_report_retry(
    pool: &PgPool,
    lease: &ReportLease,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE evaluation_reports
              SET status = CASE WHEN attempts >= 5 THEN 'failed' ELSE 'pending' END,
                  lease_token = NULL, lease_until = NULL,
                  next_attempt = clock_timestamp() + least(attempts, 12) * interval '10 minutes',
                  last_error = LEFT($3, 500), updated_at = clock_timestamp()
            WHERE id = $1 AND lease_token = $2"#,
        lease.id,
        lease.token,
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: copies the gateway's priced cost onto reports recorded before the
// audit row landed; idempotent, so it runs after every generation pass.
pub async fn update_report_costs(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE evaluation_reports r
              SET cost_microdollars = a.cost_microdollars
             FROM ai_requests a
            WHERE a.id = r.ai_request_id AND r.cost_microdollars IS NULL
              AND a.cost_microdollars IS NOT NULL"#
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

// Why: what the report job has spent on model calls since UTC midnight, the
// same cap the classifier applies.
pub async fn get_report_spend_today(pool: &PgPool, job_name: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(cost_microdollars), 0)::bigint AS "spend!"
             FROM ai_requests
            WHERE actor_kind = 'job' AND actor_id = $1
              AND created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'"#,
        job_name
    )
    .fetch_one(pool)
    .await
}

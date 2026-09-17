//! The `conversation_analyses` queue: fenced leases, completion, retry and
//! the reads a judge call needs. Discovery lives in `discovery.rs`. Every
//! statement is bound; the job never interpolates.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use super::judge::Classification;
use crate::JobError;

// Why: A claim on one conversation for the length of one judge call.
#[derive(Debug, Clone)]
pub(super) struct AnalysisLease {
    pub(super) context_id: ContextId,
    pub(super) token: String,
}

// Why: What the judge read, recorded so a conversation that grows past it is
// queued again.
#[derive(Debug, Clone, Copy)]
pub(super) struct SourceFingerprint {
    pub(super) request_count: i64,
    pub(super) last_at: Option<DateTime<Utc>>,
}

// Why: The judge's verdict plus the audit row it came from.
#[derive(Debug)]
pub(super) struct Completion<'a> {
    pub(super) classification: &'a Classification,
    pub(super) provider: &'a str,
    pub(super) model: &'a str,
    pub(super) ai_request_id: Option<&'a str>,
    pub(super) fingerprint: SourceFingerprint,
}

// Why: The conversation's rolled-up shape plus the fingerprint of the requests
// the judge is about to read.
#[derive(Debug, Clone)]
pub(super) struct SourceMeta {
    pub(super) client_kind: String,
    pub(super) model: Option<String>,
    pub(super) turn_count: i64,
    pub(super) tool_call_count: i64,
    pub(super) error_count: i64,
    pub(super) first_at: Option<DateTime<Utc>>,
    pub(super) last_at: Option<DateTime<Utc>>,
    pub(super) fingerprint: SourceFingerprint,
}

#[derive(Debug, Clone)]
pub(super) struct ConversationClassifierRepository {
    pub(super) pool: PgPool,
}

impl ConversationClassifierRepository {
    pub(super) const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) async fn claim(&self, limit: u32) -> Result<Vec<AnalysisLease>, JobError> {
        if !(1..=100).contains(&limit) {
            return Err(JobError::other("Invalid classifier batch size"));
        }
        let token = uuid::Uuid::new_v4().to_string();
        let rows = sqlx::query!(
            r#"WITH pending AS (
                   SELECT context_id FROM conversation_analyses
                   WHERE status = 'pending' AND next_attempt <= clock_timestamp()
                     AND (lease_until IS NULL OR lease_until <= clock_timestamp())
                   ORDER BY next_attempt, context_id LIMIT $1 FOR UPDATE SKIP LOCKED)
               UPDATE conversation_analyses a
               SET lease_token = $2, lease_until = clock_timestamp() + interval '5 minutes',
                   attempts = a.attempts + 1, updated_at = clock_timestamp()
               FROM pending WHERE a.context_id = pending.context_id
               RETURNING a.context_id AS "context_id!: ContextId""#,
            i64::from(limit),
            token
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|row| AnalysisLease {
                context_id: row.context_id,
                token: token.clone(),
            })
            .collect())
    }

    pub(super) async fn complete(
        &self,
        lease: &AnalysisLease,
        done: Completion<'_>,
    ) -> Result<(), JobError> {
        let c = done.classification;
        sqlx::query!(
            r#"UPDATE conversation_analyses
               SET status = 'classified', category = $3, summary = $4, tags = $5, skills_used = $6,
                   outcome = $7, confidence = $8, provider = $9, model = $10, ai_request_id = $11,
                   source_request_count = $12, source_last_at = $13,
                   classified_at = clock_timestamp(), lease_token = NULL, lease_until = NULL,
                   last_error = NULL, updated_at = clock_timestamp()
               WHERE context_id = $1 AND lease_token = $2"#,
            lease.context_id.as_str(),
            lease.token,
            c.category.as_str(),
            c.summary,
            &c.tags,
            &c.skills_observed,
            c.outcome.as_str(),
            c.confidence,
            done.provider,
            done.model,
            done.ai_request_id,
            done.fingerprint.request_count,
            done.fingerprint.last_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Why: backs off on failure; the fifth failure parks the row as `failed`
    // until someone re-queues it.
    pub(super) async fn retry(&self, lease: &AnalysisLease, error: &str) -> Result<(), JobError> {
        sqlx::query!(
            r#"UPDATE conversation_analyses
               SET status = CASE WHEN attempts >= 5 THEN 'failed' ELSE 'pending' END,
                   lease_token = NULL, lease_until = NULL,
                   next_attempt = clock_timestamp() + least(attempts, 12) * interval '5 minutes',
                   last_error = LEFT($3, 500), updated_at = clock_timestamp()
               WHERE context_id = $1 AND lease_token = $2"#,
            lease.context_id.as_str(),
            lease.token,
            error
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Why: hands leases back untouched when the run stops early (cost cap).
    pub(super) async fn release(&self, leases: &[AnalysisLease]) -> Result<(), JobError> {
        let ids: Vec<String> = leases
            .iter()
            .map(|l| l.context_id.as_str().to_owned())
            .collect();
        let Some(token) = leases.first().map(|l| l.token.as_str()) else {
            return Ok(());
        };
        sqlx::query!(
            r#"UPDATE conversation_analyses
               SET lease_token = NULL, lease_until = NULL, attempts = GREATEST(attempts - 1, 0),
                   next_attempt = clock_timestamp() + interval '1 hour', updated_at = clock_timestamp()
               WHERE context_id = ANY($1) AND lease_token = $2"#,
            &ids,
            token
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Why: what this job has spent on judge calls since UTC midnight.
    pub(super) async fn get_job_spend_today(&self, job_name: &str) -> Result<i64, JobError> {
        let spend = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(cost_microdollars), 0)::bigint AS "spend!"
               FROM ai_requests
               WHERE actor_kind = 'job' AND actor_id = $1
                 AND created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'"#,
            job_name
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(spend)
    }

    // Why: `None` when the conversation has no non-job requests left to read.
    pub(super) async fn find_source_meta(
        &self,
        context_id: &ContextId,
    ) -> Result<Option<SourceMeta>, JobError> {
        let row = sqlx::query!(
            r#"SELECT m.client_kind AS "client_kind!", m.model, m.turn_count AS "turn_count!",
                      m.tool_call_count AS "tool_call_count!", m.error_count AS "error_count!",
                      m.first_at, m.last_at,
                      (SELECT COUNT(*)::bigint FROM ai_requests ar
                        WHERE ar.context_id = $1 AND ar.actor_kind <> 'job') AS "request_count!",
                      (SELECT MAX(ar.created_at) FROM ai_requests ar
                        WHERE ar.context_id = $1 AND ar.actor_kind <> 'job') AS "last_request_at"
               FROM conversation_metrics_for(ARRAY[$1::text]) m"#,
            context_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| SourceMeta {
            client_kind: r.client_kind,
            model: r.model,
            turn_count: r.turn_count,
            tool_call_count: r.tool_call_count,
            error_count: r.error_count,
            first_at: r.first_at,
            last_at: r.last_at,
            fingerprint: SourceFingerprint {
                request_count: r.request_count,
                last_at: r.last_request_at,
            },
        }))
    }

    // Why: skills the harness hooks reported for the conversation's client
    // session, so the judge sees them alongside the transcript.
    pub(super) async fn list_hooked_skills(
        &self,
        context_id: &ContextId,
    ) -> Result<Vec<String>, JobError> {
        let rows = sqlx::query_scalar!(
            r#"SELECT s.skill AS "skill!"
               FROM conversation_skill_uses s
               WHERE s.client_session_id = (
                   SELECT ar.client_session_id FROM ai_requests ar
                   WHERE ar.context_id = $1 AND ar.client_session_id IS NOT NULL
                   ORDER BY ar.created_at DESC LIMIT 1)
               ORDER BY s.first_at"#,
            context_id.as_str()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}

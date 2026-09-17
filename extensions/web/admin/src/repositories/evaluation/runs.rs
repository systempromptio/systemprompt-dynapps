//! Judge runs: one row per attempt, prompts and response included, so a score
//! can always be read back against exactly what the model was shown.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use super::{CriterionName, Scoring};

// JSON: `evidence` is the judge's own quote list and `metrics` the snapshot of
// session numbers it was shown; both are stored as given and read back only
// for display, so they stay untyped.
#[derive(Debug, Clone)]
pub struct NewEvaluationRun {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub host: String,
    pub criterion: CriterionName,
    pub attempt: i32,
    pub succeeded: bool,
    pub score_source: Scoring,
    pub score: Option<i16>,
    pub rationale: Option<String>,
    pub evidence: serde_json::Value,
    pub metrics: serde_json::Value,
    pub provider: String,
    pub model: String,
    pub ai_request_id: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub latency_ms: Option<i32>,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response: Option<String>,
    pub error: Option<String>,
    pub config_hash: String,
    pub trigger: RunTrigger,
    pub requested_by: Option<UserId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RunTrigger {
    Automatic,
    Manual,
}

impl RunTrigger {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

pub async fn insert_run(pool: &PgPool, run: &NewEvaluationRun) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!(
        r#"INSERT INTO evaluation_runs (session_id, user_id, host, criterion, attempt, status, score_source,
              score, rationale, evidence, metrics, provider, model, ai_request_id, input_tokens, output_tokens,
              cost_microdollars, latency_ms, system_prompt, user_prompt, response, error, config_hash,
              trigger, requested_by)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25)
           RETURNING id"#,
        run.session_id.as_str(),
        run.user_id.as_str(),
        run.host,
        run.criterion.as_str(),
        run.attempt,
        if run.succeeded { "succeeded" } else { "failed" },
        run.score_source.as_str(),
        run.score,
        run.rationale,
        run.evidence,
        run.metrics,
        run.provider,
        run.model,
        run.ai_request_id,
        run.input_tokens,
        run.output_tokens,
        run.cost_microdollars,
        run.latency_ms,
        run.system_prompt,
        run.user_prompt,
        run.response,
        run.error,
        run.config_hash,
        run.trigger.as_str(),
        run.requested_by.as_ref().map(UserId::as_str),
    )
    .fetch_one(pool)
    .await
}

#[derive(Debug, Clone, Default)]
pub struct RunsFilter {
    pub criterion: Option<String>,
    pub status: Option<String>,
    pub host: Option<String>,
    pub session: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvaluationRunRow {
    pub id: String,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub host: String,
    pub criterion: String,
    pub attempt: i32,
    pub status: String,
    pub score_source: String,
    pub score: Option<i16>,
    pub model: String,
    pub provider: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub latency_ms: Option<i32>,
    pub error: Option<String>,
    pub trigger: String,
    pub requested_by: Option<UserId>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_runs_paged(
    pool: &PgPool,
    filter: &RunsFilter,
) -> Result<(Vec<EvaluationRunRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id, session_id, user_id, host, criterion, attempt, status, score_source, score, model, provider,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, error, trigger, requested_by,
                  created_at, count(*) OVER () AS "total!"
             FROM evaluation_runs
            WHERE ($1::text IS NULL OR criterion = $1)
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR host = $3)
              AND ($4::text IS NULL OR session_id = $4)
            ORDER BY created_at DESC, id
            LIMIT $5 OFFSET $6"#,
        filter.criterion,
        filter.status,
        filter.host,
        filter.session,
        filter.limit,
        filter.offset,
    )
    .fetch_all(pool)
    .await?;
    let total = rows.first().map_or(0, |r| r.total);
    let rows = rows
        .into_iter()
        .map(|r| EvaluationRunRow {
            id: r.id,
            session_id: SessionId::new(r.session_id),
            user_id: UserId::new(r.user_id),
            host: r.host,
            criterion: r.criterion,
            attempt: r.attempt,
            status: r.status,
            score_source: r.score_source,
            score: r.score,
            model: r.model,
            provider: r.provider,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            error: r.error,
            trigger: r.trigger,
            requested_by: r.requested_by.map(UserId::new),
            created_at: r.created_at,
        })
        .collect();
    Ok((rows, total))
}

// JSON: see `NewEvaluationRun` — stored as the judge returned it.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationRunDetail {
    pub row: EvaluationRunRow,
    pub rationale: Option<String>,
    pub evidence: serde_json::Value,
    pub metrics: serde_json::Value,
    pub ai_request_id: Option<String>,
    pub system_prompt: String,
    pub user_prompt: String,
    pub response: Option<String>,
    pub config_hash: String,
}

pub async fn find_run(pool: &PgPool, id: &str) -> Result<Option<EvaluationRunDetail>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id, session_id, user_id, host, criterion, attempt, status, score_source, score, model, provider,
                  input_tokens, output_tokens, cost_microdollars, latency_ms, error, trigger, requested_by,
                  created_at, rationale, evidence, metrics, ai_request_id, system_prompt, user_prompt, response, config_hash
             FROM evaluation_runs WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| EvaluationRunDetail {
        row: EvaluationRunRow {
            id: r.id,
            session_id: SessionId::new(r.session_id),
            user_id: UserId::new(r.user_id),
            host: r.host,
            criterion: r.criterion,
            attempt: r.attempt,
            status: r.status,
            score_source: r.score_source,
            score: r.score,
            model: r.model,
            provider: r.provider,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            latency_ms: r.latency_ms,
            error: r.error,
            trigger: r.trigger,
            requested_by: r.requested_by.map(UserId::new),
            created_at: r.created_at,
        },
        rationale: r.rationale,
        evidence: r.evidence,
        metrics: r.metrics,
        ai_request_id: r.ai_request_id,
        system_prompt: r.system_prompt,
        user_prompt: r.user_prompt,
        response: r.response,
        config_hash: r.config_hash,
    }))
}

// Why: the latest word on each criterion for one session — a succeeded run
// wins; otherwise the newest failure is returned so the page can say why.
#[derive(Debug, Clone, Serialize)]
pub struct SessionCriterionScore {
    pub run_id: String,
    pub criterion: String,
    pub score_source: String,
    pub score: Option<i16>,
    pub rationale: Option<String>,
    pub model: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_session_scores(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionCriterionScore>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT DISTINCT ON (criterion) id, criterion, score_source, score, rationale, model, error, created_at
             FROM evaluation_runs
            WHERE session_id = $1
            ORDER BY criterion, (status = 'succeeded') DESC, created_at DESC"#,
        session_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SessionCriterionScore {
            run_id: r.id,
            criterion: r.criterion,
            score_source: r.score_source,
            score: r.score,
            rationale: r.rationale,
            model: r.model,
            error: r.error,
            created_at: r.created_at,
        })
        .collect())
}

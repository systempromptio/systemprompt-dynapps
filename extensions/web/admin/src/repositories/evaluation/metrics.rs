//! The session's own numbers: what its requests cost and how long they took.
//!
//! Only the person's conversational turns count. Judge calls (`actor_kind =
//! 'job'`) and probes are excluded so a session is never scored on the spend
//! of scoring it.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, Default, Serialize)]
pub struct SessionMetrics {
    pub requests: i64,
    pub completed: i64,
    pub failed: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_microdollars: i64,
    pub avg_latency_ms: Option<f64>,
    pub max_latency_ms: Option<i32>,
    pub models: Vec<SessionModelShare>,
    pub duration_minutes: f64,
    pub user_prompts: i64,
    pub tool_calls: i64,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionModelShare {
    pub model: String,
    pub requests: i64,
    pub cost_microdollars: i64,
}

pub async fn get_session_metrics(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
) -> Result<SessionMetrics, sqlx::Error> {
    let requests = sqlx::query!(
        r#"SELECT count(*) AS "requests!",
                  count(*) FILTER (WHERE status = 'completed') AS "completed!",
                  count(*) FILTER (WHERE status = 'failed') AS "failed!",
                  COALESCE(sum(input_tokens), 0)::BIGINT AS "input_tokens!",
                  COALESCE(sum(output_tokens), 0)::BIGINT AS "output_tokens!",
                  COALESCE(sum(cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!",
                  COALESCE(sum(cache_creation_tokens), 0)::BIGINT AS "cache_creation_tokens!",
                  COALESCE(sum(cost_microdollars), 0)::BIGINT AS "cost_microdollars!",
                  avg(latency_ms)::FLOAT8 AS avg_latency_ms,
                  max(latency_ms) AS max_latency_ms
             FROM ai_requests
            WHERE user_id = $1 AND client_session_id = $2
              AND actor_kind = 'user' AND request_kind = 'turn'"#,
        user_id.as_str(),
        session_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    let models = sqlx::query!(
        r#"SELECT COALESCE(model, requested_model, 'unknown') AS "model!", count(*) AS "requests!", COALESCE(sum(cost_microdollars), 0)::BIGINT AS "cost!"
             FROM ai_requests
            WHERE user_id = $1 AND client_session_id = $2
              AND actor_kind = 'user' AND request_kind = 'turn'
            GROUP BY 1 ORDER BY "cost!" DESC"#,
        user_id.as_str(),
        session_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    let session = sqlx::query!(
        r#"SELECT COALESCE(s.user_prompts, s.prompts)::BIGINT AS "user_prompts!",
                  s.tool_uses AS "tool_calls!", s.errors AS "errors!",
                  EXTRACT(EPOCH FROM (COALESCE(s.ended_at, s.updated_at) - COALESCE(s.started_at, s.created_at)))::FLOAT8 AS seconds
             FROM plugin_session_summaries s WHERE s.session_id = $1"#,
        session_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    let (user_prompts, tool_calls, errors, seconds) = session.map_or((0, 0, 0, 0.0), |s| {
        (
            s.user_prompts,
            s.tool_calls,
            s.errors,
            s.seconds.unwrap_or(0.0).max(0.0),
        )
    });
    Ok(SessionMetrics {
        requests: requests.requests,
        completed: requests.completed,
        failed: requests.failed,
        input_tokens: requests.input_tokens,
        output_tokens: requests.output_tokens,
        cache_read_tokens: requests.cache_read_tokens,
        cache_creation_tokens: requests.cache_creation_tokens,
        cost_microdollars: requests.cost_microdollars,
        avg_latency_ms: requests.avg_latency_ms,
        max_latency_ms: requests.max_latency_ms,
        models: models
            .into_iter()
            .map(|m| SessionModelShare {
                model: m.model,
                requests: m.requests,
                cost_microdollars: m.cost,
            })
            .collect(),
        duration_minutes: seconds / 60.0,
        user_prompts,
        tool_calls,
        errors,
    })
}

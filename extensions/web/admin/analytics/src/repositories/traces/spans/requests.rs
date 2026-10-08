//! Gateway-request spans: one model span per `ai_requests` row, ending at
//! `completed_at` or, while still open, at start plus the reported latency.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use super::{format_identity, request_status};
use crate::repositories::traces::{Span, SpanKind};

pub(super) async fn list_request_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let requests = sqlx::query!(
        r#"SELECT
            r.id              AS "id!",
            r.trace_id,
            r.context_id      AS "context_id!: ContextId",
            r.provider,
            r.model,
            r.status          AS "status!",
            r.latency_ms,
            r.input_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.error_message,
            r.client_kind     AS "client_kind!",
            r.cost_microdollars AS "cost_microdollars!",
            r.created_at      AS "created_at!",
            r.completed_at,
            r.user_id         AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, r.user_id) AS "user_label!"
        FROM ai_requests r
        LEFT JOIN users u ON u.id = r.user_id
        WHERE r.session_id = $1 OR r.client_session_id = $1
           OR (r.session_id IS NULL AND r.trace_id = $1)
        ORDER BY r.created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(requests
        .into_iter()
        .map(|r| {
            let started = r.created_at;
            let ended = r.completed_at.unwrap_or_else(|| {
                started
                    + chrono::Duration::milliseconds(i64::from(r.latency_ms.unwrap_or(0)).max(0))
            });
            let name = match (r.provider.as_deref(), r.model.as_deref()) {
                (Some(p), Some(m)) => format!("{p}/{m}"),
                _ => "—".to_owned(),
            };
            Span {
                id: r.id,
                kind: SpanKind::Model,
                name,
                started_at: started,
                ended_at: ended,
                duration_ms: (ended - started).num_milliseconds().max(0),
                status: request_status(&r.status),
                identity_label: Some(format_identity(&r.user_label, None, None)),
                provider: r.provider,
                model: r.model,
                cost_microdollars: Some(r.cost_microdollars),
                latency_ms: r.latency_ms.map(i64::from),
                input_tokens: r.input_tokens.map(i64::from),
                output_tokens: r.output_tokens.map(i64::from),
                cache_read_tokens: r.cache_read_tokens.map(i64::from),
                error_message: r.error_message.filter(|e| !e.is_empty()),
                trace_id: r.trace_id,
                context_id: Some(r.context_id),
                user_id: r.user_id,
                client_kind: Some(r.client_kind),
                policy: None,
                decision: None,
                reason: None,
                stages: Vec::new(),
                artifact_id: None,
            }
        })
        .collect())
}

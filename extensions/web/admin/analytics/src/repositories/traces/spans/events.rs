//! Hook-event spans from `plugin_usage_events`: a `Spawn` event is a spawn
//! span, anything else a tool span, and the event type rides in `policy`.

use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, SessionId, UserId};

use super::format_identity;
use crate::repositories::traces::{Span, SpanKind, SpanStatus};

pub(super) async fn list_event_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let events = sqlx::query!(
        r#"SELECT
            e.id              AS "id!",
            e.event_type      AS "event_type!",
            e.tool_name,
            e.description,
            e.user_id         AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, e.user_id) AS "user_label!",
            a.artifact_id     AS "artifact_id?: ArtifactId",
            e.created_at      AS "created_at!"
        FROM plugin_usage_events e
        LEFT JOIN users u ON u.id = e.user_id
        LEFT JOIN mcp_artifacts a ON a.mcp_execution_id = e.mcp_execution_id
        WHERE e.session_id = $1
           OR e.session_id IN (
               SELECT DISTINCT client_session_id FROM ai_requests
               WHERE session_id = $1 AND client_session_id IS NOT NULL)
        ORDER BY e.created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(events
        .into_iter()
        .map(|e| {
            let kind = if e.event_type.contains("Spawn") {
                SpanKind::Spawn
            } else {
                SpanKind::Tool
            };
            let status = if e.event_type.contains("Failure") || e.event_type.contains("Error") {
                SpanStatus::Error
            } else {
                SpanStatus::Ok
            };
            Span {
                id: e.id,
                kind,
                name: e.tool_name.unwrap_or_else(|| e.event_type.clone()),
                started_at: e.created_at,
                ended_at: e.created_at,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(&e.user_label, None, None)),
                provider: None,
                model: None,
                cost_microdollars: None,
                latency_ms: None,
                input_tokens: None,
                output_tokens: None,
                cache_read_tokens: None,
                error_message: None,
                trace_id: None,
                context_id: None,
                user_id: e.user_id,
                client_kind: None,
                policy: Some(e.event_type),
                decision: None,
                reason: e.description.filter(|d| !d.is_empty()),
                stages: Vec::new(),
                artifact_id: e.artifact_id,
            }
        })
        .collect())
}

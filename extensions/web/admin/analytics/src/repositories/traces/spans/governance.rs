//! Governance-decision spans: one zero-length span per decision, with the
//! chain audit's stages attached. Decisions keyed on Claude Code's uuid or the
//! request's `trace_id` are folded into the bridge session.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use super::{format_identity, parse_stages};
use crate::repositories::traces::{Span, SpanKind, SpanStatus};

pub(super) async fn list_governance_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let decisions = sqlx::query!(
        r#"SELECT
            d.id              AS "id!",
            d.tool_name       AS "tool_name!",
            d.policy          AS "policy!",
            d.decision        AS "decision!",
            d.reason          AS "reason!",
            d.agent_id,
            d.agent_scope,
            d.trace_id,
            d.context_id      AS "context_id!: ContextId",
            d.client_id,
            d.evaluated_rules,
            d.user_id         AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, d.user_id) AS "user_label!",
            d.created_at      AS "created_at!"
        FROM governance_decisions d
        LEFT JOIN users u ON u.id = d.user_id
        WHERE d.session_id = $1
           OR d.session_id IN (
               SELECT DISTINCT trace_id FROM ai_requests
               WHERE session_id = $1 AND trace_id IS NOT NULL)
           OR d.session_id IN (
               SELECT DISTINCT client_session_id FROM ai_requests
               WHERE session_id = $1 AND client_session_id IS NOT NULL)
        ORDER BY d.created_at ASC"#,
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(decisions
        .into_iter()
        .map(|d| {
            let status = match d.decision.as_str() {
                "deny" => SpanStatus::Deny,
                "warn" => SpanStatus::Warn,
                _ => SpanStatus::Ok,
            };
            let stages = parse_stages(d.evaluated_rules);
            Span {
                id: d.id,
                kind: SpanKind::Governance,
                name: d.tool_name,
                started_at: d.created_at,
                ended_at: d.created_at,
                duration_ms: 0,
                status,
                identity_label: Some(format_identity(
                    &d.user_label,
                    d.agent_id.as_deref(),
                    d.agent_scope.as_deref(),
                )),
                provider: None,
                model: None,
                cost_microdollars: None,
                latency_ms: None,
                input_tokens: None,
                output_tokens: None,
                cache_read_tokens: None,
                error_message: None,
                trace_id: d.trace_id,
                context_id: Some(d.context_id),
                user_id: d.user_id,
                client_kind: d.client_id,
                policy: Some(d.policy),
                decision: Some(d.decision),
                reason: Some(d.reason).filter(|r| !r.is_empty()),
                stages,
                artifact_id: None,
            }
        })
        .collect())
}

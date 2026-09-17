//! Per-session spans, unioned and normalised across the governance,
//! gateway-request, and tool-event tables.
//!
//! Every span names the person behind it by display label, not by id, and a
//! model span carries the tokens, cache reads and error it reported; the
//! trace page folds a request's governance decisions under the request by
//! the shared `trace_id`, so nothing here pre-groups.

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, ContextId, SessionId, UserId};

use super::{PolicyStage, Span, SpanKind, SpanStatus};

// JSON: `governance_decisions.evaluated_rules` is written by each policy in
// its own shape; only the chain audit carries a `chain` array of stages.
#[derive(Debug, Deserialize)]
struct ChainAudit {
    #[serde(default)]
    chain: Vec<ChainStage>,
}

#[derive(Debug, Deserialize)]
struct ChainStage {
    policy_id: String,
    result: String,
    #[serde(default)]
    detail: String,
    #[serde(default)]
    duration_ms: f64,
}

async fn list_governance_spans(
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

async fn list_request_spans(
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
        WHERE r.session_id = $1
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

async fn list_event_spans(pool: &PgPool, session_id: &SessionId) -> Result<Vec<Span>, sqlx::Error> {
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

pub async fn list_trace_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let mut spans = list_governance_spans(pool, session_id).await?;
    spans.extend(list_request_spans(pool, session_id).await?);
    spans.extend(list_event_spans(pool, session_id).await?);
    spans.sort_by_key(|s| s.started_at);
    Ok(spans)
}

// JSON: the audit column is typed here, at the one place it is read.
fn parse_stages(evaluated_rules: Option<serde_json::Value>) -> Vec<PolicyStage> {
    evaluated_rules
        .and_then(|v| serde_json::from_value::<ChainAudit>(v).ok())
        .map(|a| {
            a.chain
                .into_iter()
                .map(|c| PolicyStage {
                    policy_id: c.policy_id,
                    result: c.result,
                    detail: c.detail,
                    duration_ms: c.duration_ms,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn request_status(status: &str) -> SpanStatus {
    match status {
        "ok" | "success" | "completed" => SpanStatus::Ok,
        "pending" => SpanStatus::Pending,
        "rejected" => SpanStatus::Rejected,
        _ => SpanStatus::Error,
    }
}

fn format_identity(user: &str, agent: Option<&str>, scope: Option<&str>) -> String {
    match (agent, scope) {
        (Some(a), Some(s)) => format!("{user} · {a} ({s})"),
        (Some(a), None) => format!("{user} · {a}"),
        _ => user.to_owned(),
    }
}

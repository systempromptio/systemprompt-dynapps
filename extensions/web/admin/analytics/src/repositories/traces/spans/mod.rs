//! Per-session spans, unioned and normalised across the governance,
//! gateway-request, and tool-event tables; the tool-call-ledger source is
//! the sibling `ledger_spans` module.
//!
//! Every span names the person behind it by display label, not by id, and a
//! model span carries the tokens, cache reads and error it reported; the
//! trace page folds a request's governance decisions under the request by
//! the shared `trace_id`, so nothing here pre-groups.

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use super::{PolicyStage, Span, SpanStatus};

mod events;
mod governance;
mod requests;

// JSON: variable-shape `evaluated_rules`; only the chain audit carries `chain`.
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

pub async fn list_trace_spans(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<Span>, sqlx::Error> {
    let mut spans = governance::list_governance_spans(pool, session_id).await?;
    spans.extend(requests::list_request_spans(pool, session_id).await?);
    spans.extend(events::list_event_spans(pool, session_id).await?);
    spans.extend(super::ledger_spans::list_ledger_spans(pool, session_id).await?);
    spans.sort_by_key(|s| s.started_at);
    Ok(spans)
}

// JSON: variable-shape audit column, typed here at the one place it is read.
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

pub(super) fn format_identity(user: &str, agent: Option<&str>, scope: Option<&str>) -> String {
    match (agent, scope) {
        (Some(a), Some(s)) => format!("{user} · {a} ({s})"),
        (Some(a), None) => format!("{user} · {a}"),
        _ => user.to_owned(),
    }
}

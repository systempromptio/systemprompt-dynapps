//! The provenance card on `/admin/artifacts/{artifact_id}`: which capture
//! path saw the call, how it was correlated, and the request, session and
//! governance rows it links to.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin_core::ssr::entity_urls::{request_detail_url, session_detail_url};
use systemprompt_web_admin_core::ssr::format::{
    client_label, format_duration_ms, local_time, short_id,
};

use crate::repositories::analytics::artifacts::ArtifactDetail;

fn source_label(source: &str) -> String {
    match source {
        "in_process" => "In-process".to_owned(),
        "proxy" => "Proxy tap".to_owned(),
        "gateway" => "Gateway history".to_owned(),
        "hook_claude_code" => "Claude Code hook".to_owned(),
        "hook_opencode" => "OpenCode hook".to_owned(),
        other => other.to_owned(),
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ProvenanceView {
    source: String,
    source_label: String,
    last_seen_source: Option<String>,
    last_seen_label: Option<String>,
    correlation: String,
    correlation_tone: &'static str,
    state_label: String,
    state_tone: &'static str,
    client_label: Option<String>,
    ai_tool_call_id: Option<String>,
    mcp_execution_id: String,
    execution_status: Option<String>,
    execution_time_display: Option<String>,
    error_message: Option<String>,
    intended_at: Option<String>,
    request_id: Option<String>,
    request_url: Option<String>,
    session_id: Option<SessionId>,
    session_url: Option<String>,
    trace_id: Option<String>,
    user_id: Option<UserId>,
    user_label: String,
    governance_decision_id: Option<String>,
    governance_decision: Option<String>,
    governance_url: Option<String>,
}

pub(super) fn provenance_view(row: &mut ArtifactDetail) -> ProvenanceView {
    let correlation = row.correlation.take().unwrap_or_else(|| "exact".to_owned());
    let (state_label, state_tone) = if row.is_error {
        ("failed".to_owned(), "err")
    } else if row.request_id.is_some() {
        ("executed".to_owned(), "ok")
    } else {
        ("unattested".to_owned(), "warn")
    };
    let user_label = row
        .user_label
        .take()
        .or_else(|| row.user_id.as_ref().map(|u| short_id(u.as_str())))
        .unwrap_or_else(|| "\u{2014}".to_owned());
    let source = std::mem::take(&mut row.source);
    let last_seen_source = row.last_seen_source.take();
    let request_id = row.request_id.take();
    let session_id = row.session_id.take();
    let governance_decision_id = row.governance_decision_id.take();
    ProvenanceView {
        source_label: source_label(&source),
        source,
        last_seen_label: last_seen_source.as_deref().map(source_label),
        last_seen_source,
        correlation_tone: if correlation == "inferred" {
            "warn"
        } else {
            "ok"
        },
        correlation,
        state_label,
        state_tone,
        client_label: row.client_kind.as_deref().map(client_label),
        ai_tool_call_id: row.ai_tool_call_id.take(),
        mcp_execution_id: row.mcp_execution_id.to_string(),
        execution_status: row.execution_status.take(),
        execution_time_display: row
            .execution_time_ms
            .map(|ms| format_duration_ms(i64::from(ms))),
        error_message: row.error_message.take(),
        intended_at: row.intended_at.map(local_time),
        request_url: request_id
            .as_deref()
            .map(|id| request_detail_url(&systemprompt::identifiers::AiRequestId::new(id))),
        request_id,
        session_url: session_id.as_ref().map(session_detail_url),
        session_id,
        trace_id: row.trace_id.take(),
        user_id: row.user_id.take(),
        user_label,
        governance_decision: row.governance_decision.take(),
        governance_url: governance_decision_id
            .as_ref()
            .map(|id| format!("/admin/governance/decisions/{id}")),
        governance_decision_id,
    }
}

//! `/admin/artifacts/{artifact_id}` — one artifact: what it is, where it was
//! seen, what the scanners found, and every row of the same call.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url};
use crate::handlers::ssr::format::{client_label, format_duration_ms, local_time, short_id};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::ssr_tools::help::artifacts_help;
use crate::handlers::ssr::ssr_tools::rows::{format_bytes, kind_icon, kind_label};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::artifacts::{
    ArtifactDetail, ArtifactFindingRow, find_artifact,
};

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
struct ArtifactDetailContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    artifact: ArtifactView,
    provenance: ProvenanceView,
    findings: Vec<FindingView>,
    finding_count: usize,
    has_findings: bool,
    body_pretty: Option<String>,
    body_missing: bool,
    preview_url: Option<String>,
    back_url: &'static str,
    help: HelpView,
}

#[derive(Debug, Serialize)]
struct ArtifactView {
    artifact_id: ArtifactId,
    artifact_id_short: String,
    artifact_title: String,
    tool_name: String,
    server_name: String,
    artifact_type: String,
    kind_label: &'static str,
    kind_icon: &'static str,
    input_summary: Option<String>,
    is_builtin: bool,
    is_structured: bool,
    has_ui_resource: bool,
    is_error: bool,
    bytes_display: String,
    payload_sha256: Option<String>,
    secret_redactions: i32,
    created_at: String,
    skill: Option<String>,
    skill_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct ProvenanceView {
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

#[derive(Debug, Serialize)]
struct FindingView {
    phase: String,
    severity: String,
    severity_tone: &'static str,
    category: String,
    scanner: String,
    path: Option<String>,
    excerpt: Option<String>,
    redacted: bool,
    created_at: String,
}

pub(crate) async fn artifact_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(artifact_id): Path<ArtifactId>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let detail = find_artifact(&pool, &artifact_id)
        .await
        .map_err(AdminError::from)?
        .ok_or_else(|| AdminError::NotFound(format!("Artifact {artifact_id} not found")))?;

    let ctx = build_context(detail);
    Ok(render_typed_page(
        &shell.engine,
        "artifact-detail",
        &ctx,
        &shell.user,
        &shell.marketplace,
    ))
}

fn build_context(mut detail: ArtifactDetail) -> ArtifactDetailContext {
    let id_short = short_id(detail.artifact_id.as_str());
    let findings = std::mem::take(&mut detail.findings);
    let body = detail.body.take();
    let body_pretty = body
        .as_ref()
        .and_then(|body| serde_json::to_string_pretty(body).ok());
    let preview_url = (detail.is_structured && body.is_some())
        .then(|| format!("/admin/artifacts/{}/preview", detail.artifact_id));
    let provenance = provenance_view(&mut detail);

    ArtifactDetailContext {
        page: "artifact-detail",
        title: format!("Artifact {id_short}"),
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::link("Tools & artifacts", "/admin/tools"),
            BreadcrumbView::link("Artifacts", "/admin/artifacts"),
            BreadcrumbView::current(id_short),
        ],
        artifact: artifact_view(detail),
        provenance,
        finding_count: findings.len(),
        has_findings: !findings.is_empty(),
        findings: findings.into_iter().map(finding_view).collect(),
        body_missing: body.is_none(),
        body_pretty,
        preview_url,
        back_url: "/admin/artifacts",
        help: artifacts_help(),
    }
}

fn artifact_view(detail: ArtifactDetail) -> ArtifactView {
    let ArtifactDetail {
        artifact_id,
        tool_name,
        artifact_title,
        input_summary,
        server_name,
        artifact_type,
        artifact_kind,
        is_builtin,
        is_structured,
        has_ui_resource,
        is_error,
        payload_bytes,
        payload_sha256,
        secret_redactions,
        created_at,
        skill,
        ..
    } = detail;
    let tool_name = tool_name.unwrap_or_else(|| "unknown".to_owned());
    ArtifactView {
        artifact_id_short: short_id(artifact_id.as_str()),
        artifact_id,
        artifact_title: artifact_title
            .or_else(|| input_summary.clone())
            .unwrap_or_else(|| tool_name.clone()),
        tool_name,
        server_name,
        artifact_type,
        kind_label: kind_label(artifact_kind.as_deref()),
        kind_icon: kind_icon(artifact_kind.as_deref()),
        input_summary,
        is_builtin,
        is_structured,
        has_ui_resource,
        is_error,
        bytes_display: format_bytes(i64::from(payload_bytes.unwrap_or(0))),
        payload_sha256,
        secret_redactions,
        created_at: local_time(created_at),
        skill_url: skill
            .as_ref()
            .map(|s| format!("/admin/artifacts?skill={}", urlencoding::encode(s))),
        skill,
    }
}

fn provenance_view(row: &mut ArtifactDetail) -> ProvenanceView {
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

fn finding_view(f: ArtifactFindingRow) -> FindingView {
    FindingView {
        severity_tone: match f.severity.as_str() {
            "high" => "err",
            "medium" => "warn",
            _ => "info",
        },
        phase: f.phase,
        severity: f.severity,
        category: f.category,
        scanner: f.scanner,
        path: f.path,
        excerpt: f.excerpt,
        redacted: f.redacted,
        created_at: local_time(f.created_at),
    }
}

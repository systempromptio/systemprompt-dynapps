//! One list row per artifact.

use serde::Serialize;
use systemprompt::identifiers::{ArtifactId, SessionId, UserId};

use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url};
use crate::handlers::ssr::format::{client_label, local_time, relative_time, short_id};
use crate::repositories::analytics::artifacts::ArtifactRow;

use super::context::format_bytes;

#[derive(Debug, Serialize)]
pub(super) struct ArtifactRowView {
    pub(super) artifact_id: ArtifactId,
    pub(super) artifact_id_short: String,
    pub(super) detail_url: String,
    // Why: not `title` — the layout's `title=` hash parameter shadows a field
    // of that name at every depth of the template.
    pub(super) artifact_title: String,
    pub(super) tool_name: String,
    pub(super) server_name: String,
    pub(super) artifact_type: String,
    pub(super) skill: Option<String>,
    pub(super) skill_url: Option<String>,
    pub(super) user_id: Option<UserId>,
    pub(super) user_label: String,
    pub(super) session_id: Option<SessionId>,
    pub(super) session_url: Option<String>,
    pub(super) request_url: Option<String>,
    pub(super) source: String,
    pub(super) source_label: String,
    pub(super) client_label: Option<String>,
    pub(super) correlation: String,
    pub(super) correlation_tone: &'static str,
    pub(super) state_label: String,
    pub(super) state_tone: &'static str,
    pub(super) is_structured: bool,
    pub(super) has_ui_resource: bool,
    pub(super) is_error: bool,
    pub(super) finding_count: i64,
    pub(super) secret_redactions: i32,
    pub(super) bytes_display: String,
    pub(super) created_at: String,
    pub(super) created_relative: String,
}

pub(super) fn source_label(source: &str) -> String {
    match source {
        "in_process" => "In-process".to_owned(),
        "proxy" => "Proxy tap".to_owned(),
        "gateway" => "Gateway history".to_owned(),
        "hook_claude_code" => "Claude Code hook".to_owned(),
        "hook_opencode" => "OpenCode hook".to_owned(),
        other => other.to_owned(),
    }
}

pub(super) fn artifact_row(row: &ArtifactRow) -> ArtifactRowView {
    let tool_name = row
        .tool_name
        .clone()
        .unwrap_or_else(|| "unknown".to_owned());
    let correlation = row
        .correlation
        .clone()
        .unwrap_or_else(|| "exact".to_owned());
    let (state_label, state_tone) = if row.is_error {
        ("failed".to_owned(), "err")
    } else if row.request_id.is_some() {
        ("executed".to_owned(), "ok")
    } else {
        ("unattested".to_owned(), "warn")
    };
    ArtifactRowView {
        artifact_id: row.artifact_id.clone(),
        artifact_id_short: short_id(row.artifact_id.as_str()),
        detail_url: format!("/admin/artifacts/{}", row.artifact_id),
        artifact_title: row.title.clone().unwrap_or_else(|| tool_name.clone()),
        tool_name,
        server_name: row.server_name.clone(),
        artifact_type: row.artifact_type.clone(),
        skill: row.skill.clone(),
        skill_url: row
            .skill
            .as_ref()
            .map(|s| format!("/admin/artifacts?skill={}", urlencoding::encode(s))),
        user_id: row.user_id.clone(),
        user_label: row
            .user_label
            .clone()
            .or_else(|| row.user_id.as_ref().map(|u| short_id(u.as_str())))
            .unwrap_or_else(|| "\u{2014}".to_owned()),
        session_id: row.session_id.clone(),
        session_url: row.session_id.as_ref().map(session_detail_url),
        request_url: row
            .request_id
            .as_deref()
            .map(|id| request_detail_url(&systemprompt::identifiers::AiRequestId::new(id))),
        source_label: source_label(&row.source),
        source: row.source.clone(),
        client_label: row.client_kind.as_deref().map(client_label),
        correlation_tone: if correlation == "inferred" {
            "warn"
        } else {
            "ok"
        },
        correlation,
        state_label,
        state_tone,
        is_structured: row.is_structured,
        has_ui_resource: row.has_ui_resource,
        is_error: row.is_error,
        finding_count: row.finding_count,
        secret_redactions: row.secret_redactions,
        bytes_display: format_bytes(i64::from(row.payload_bytes.unwrap_or(0))),
        created_at: local_time(row.created_at),
        created_relative: relative_time(row.created_at),
    }
}

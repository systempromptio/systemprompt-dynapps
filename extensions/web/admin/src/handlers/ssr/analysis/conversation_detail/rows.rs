//! Row views for the conversation detail's tables — turns, tool calls,
//! decisions, skills, safety findings — and the judge's card.

use super::views::{
    ConversationSkillView, DecisionView, JudgeView, LedgerCallView, LedgerTurnView, SafetyView,
};
use crate::handlers::ssr::analysis::conversations::summary::{
    category_label, outcome_label, outcome_tone,
};
use crate::handlers::ssr::analysis::conversations::view::SkillChipView;
use crate::handlers::ssr::analysis::tone::{latency_tone, safety_tone, score_tone};
use crate::handlers::ssr::analysis_urls::{analysis_skill_url, analysis_version_url};
use crate::handlers::ssr::format::{
    format_cost, format_duration_ms, format_token_total, local_time,
};
use crate::handlers::ssr::ssr_tools::rows::{format_bytes, kind_icon, kind_label};
use crate::repositories::analysis::conversations::ConversationFactRow;
use crate::repositories::analysis::conversations::planes::{
    ConversationDecisionRow, ConversationSafetyRow, ConversationSkillRow, ConversationToolCallRow,
    ConversationTurnRow,
};

pub(super) fn turn_view((index, t): (usize, ConversationTurnRow)) -> LedgerTurnView {
    let is_turn = t.effective_kind == "turn";
    let routed = t.requested_model.filter(|m| Some(m) != t.model.as_ref());
    LedgerTurnView {
        index: index + 1,
        href: format!("/admin/requests/{}", urlencoding::encode(&t.request_id)),
        at: t.created_at.format("%H:%M:%S").to_string(),
        kind: match t.effective_kind.as_str() {
            "turn" => "turn",
            "probe" => "probe",
            _ => "side call",
        },
        is_turn,
        status_tone: match t.status.as_str() {
            "completed" => "ok",
            "failed" => "err",
            "rejected" => "warn",
            _ => "muted",
        },
        input_display: format_token_total(t.input_tokens),
        output_display: format_token_total(t.output_tokens),
        cache_display: format_token_total(t.cache_read_tokens + t.cache_creation_tokens),
        reasoning_display: format_token_total(t.reasoning_tokens),
        cost_display: format_cost(t.cost_microdollars),
        latency_display: t
            .latency_ms
            .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
        latency_tone: latency_tone(t.latency_ms.map(f64::from)),
        streaming: t.is_streaming,
        tools: t.tool_calls,
        tool_names: t.tool_names.join(", "),
        safety: t.safety_findings,
        safety_tone: safety_tone(t.safety_findings, t.safety_blocked),
        request_id: t.request_id,
        model: t.model.unwrap_or_else(|| "—".to_owned()),
        routed,
        status: t.status,
        finish_reason: t.finish_reason.unwrap_or_default(),
        error_message: t.error_message.filter(|m| !m.is_empty()),
    }
}

fn call_state_tone(failed: bool, state: &str) -> &'static str {
    match (failed, state) {
        (true, _) => "err",
        (_, "executed") => "ok",
        (_, "intended") => "warn",
        _ => "muted",
    }
}

pub(super) fn tool_view(t: ConversationToolCallRow) -> LedgerCallView {
    let tool_name = t.tool_name.unwrap_or_else(|| "—".to_owned());
    let input = t.input_summary.unwrap_or_default();
    let kind = t.artifact_kind.as_deref();
    let failed = t.is_error == Some(true)
        || matches!(t.execution_status.as_deref(), Some("failed" | "timeout"));
    let previewable = matches!(kind, Some("card" | "ui" | "body")) && t.is_structured && !failed;
    let artifact_href = t
        .artifact_id
        .as_ref()
        .filter(|_| kind.is_some())
        .map(|id| format!("/admin/artifacts/{}", urlencoding::encode(id.as_str())));
    let preview_href = t.artifact_id.as_ref().filter(|_| previewable).map(|id| {
        format!(
            "/admin/artifacts/{}/preview",
            urlencoding::encode(id.as_str())
        )
    });
    let is_path = kind == Some("file") || input.starts_with('/');
    let (artifact_icon, artifact_label) = (kind_icon(kind), kind_label(kind));
    let state_tone = call_state_tone(failed, &t.state);
    let artifact_title = t
        .artifact_title
        .filter(|title| !title.is_empty() && title != &tool_name)
        .or_else(|| (!input.is_empty()).then(|| input.clone()))
        .or(t.artifact_type);
    LedgerCallView {
        tool_icon: if t.is_builtin { "terminal" } else { "plug" },
        is_builtin: t.is_builtin,
        server_name: t.server_name.unwrap_or_default(),
        is_path,
        state_tone,
        state: if failed { "failed".to_owned() } else { t.state },
        source: t.source.unwrap_or_default(),
        status: t.execution_status.unwrap_or_default(),
        duration_display: t
            .execution_time_ms
            .map_or_else(|| "—".to_owned(), format_duration_ms),
        at: t.occurred_at.map(local_time).unwrap_or_default(),
        artifact_icon,
        artifact_label,
        artifact_title,
        preview_href,
        artifact_href,
        artifact_kind: t.artifact_kind,
        bytes_display: format_bytes(i64::from(t.payload_bytes.unwrap_or(0))),
        error_message: t.error_message.filter(|m| !m.is_empty()),
        input_summary: input,
        tool_name,
    }
}

pub(super) fn decision_view(d: ConversationDecisionRow) -> DecisionView {
    DecisionView {
        tone: match d.decision.as_str() {
            "allow" => "ok",
            "warn" => "warn",
            "deny" => "err",
            _ => "muted",
        },
        tool_name: d.tool_name,
        decision: d.decision,
        policy: d.policy,
        reason: d.reason,
        plugin: d
            .plugin_id
            .map(|p| p.as_str().to_owned())
            .unwrap_or_default(),
        at: local_time(d.created_at),
    }
}

pub(super) fn skill_view(s: ConversationSkillRow) -> ConversationSkillView {
    let version_href = s
        .marketplace_id
        .as_ref()
        .map(|m| analysis_version_url(m, s.marketplace_hash.as_deref()));
    ConversationSkillView {
        chip: SkillChipView {
            href: analysis_skill_url(&s.skill),
            name: s.skill,
            hooked: true,
        },
        plugin: s
            .plugin_id
            .map(|p| p.as_str().to_owned())
            .unwrap_or_default(),
        marketplace: s
            .marketplace_id
            .map(|m| m.as_str().to_owned())
            .unwrap_or_default(),
        version_short: s
            .marketplace_hash
            .as_deref()
            .map_or_else(|| "—".to_owned(), |h| h[..h.len().min(12)].to_owned()),
        version_href,
        invocations: s.invocations,
        first_at: local_time(s.first_invoked_at),
    }
}

pub(super) fn safety_view(s: ConversationSafetyRow) -> SafetyView {
    SafetyView {
        tone: match (s.blocked, s.severity.as_str()) {
            (true, _) | (_, "critical") => "err",
            (_, "high" | "medium") => "warn",
            _ => "muted",
        },
        request_href: format!("/admin/requests/{}", urlencoding::encode(&s.request_id)),
        phase: s.phase,
        category: s.category,
        severity: s.severity,
        scanner: s.scanner,
        blocked: s.blocked,
        at: local_time(s.created_at),
    }
}

pub(super) fn judge_view(f: &ConversationFactRow, judge_url: String) -> JudgeView {
    JudgeView {
        judged: f.completion.is_some(),
        pending: f.completion.is_none() && f.judge_status.as_deref() == Some("pending"),
        completion_display: f
            .completion
            .map_or_else(|| "—".to_owned(), |c| c.to_string()),
        tone: f.completion.map_or("muted", score_tone),
        category_label: f
            .category
            .as_deref()
            .map_or("Not judged yet", category_label),
        outcome: f.outcome.as_deref().map_or("—", outcome_label),
        outcome_tone: f.outcome.as_deref().map_or("muted", outcome_tone),
        summary: f.summary.clone().unwrap_or_default(),
        rationale: f.completion_rationale.clone().unwrap_or_default(),
        tags: f.tags.clone(),
        judged_at: f.classified_at.map(local_time).unwrap_or_default(),
        model: f.judge_model.clone().unwrap_or_default(),
        cost_display: f
            .judge_cost_microdollars
            .map_or_else(|| "—".to_owned(), format_cost),
        tokens_display: format_token_total(f.judge_tokens),
        trigger: f.judge_trigger.clone().unwrap_or_default(),
        judge_url,
    }
}

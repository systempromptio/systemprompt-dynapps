//! Row and tile shaping for the governance page.
//!
//! The repository answers in domain rows; the template reads strings and URLs.
//! Everything in between — badge tones, the stage a policy belongs to, the
//! reason truncation — happens here, so the template holds no logic and the
//! handler holds no formatting.

use serde::Serialize;
use systemprompt::identifiers::{PluginId, UserId};

use systemprompt_web_shared::format::truncate_ellipsis;

use crate::handlers::ssr::format::local_time;
use crate::repositories::governance::decision_calls::{ChainEvaluation, DecisionCallRow};
use crate::repositories::governance::findings::SafetyFindingLogRow;
use crate::repositories::governance::hook_events::RecentHookEvent;

const REASON_CHARS: usize = 90;

#[derive(Debug, Serialize)]
pub(super) struct ChainPill {
    pub(super) policy: String,
    pub(super) tone: &'static str,
    pub(super) title: String,
    pub(super) href: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DecisionRow {
    pub(super) created_at: String,
    pub(super) span: String,
    pub(super) decision: String,
    pub(super) tone: &'static str,
    pub(super) attention: bool,
    pub(super) target: String,
    pub(super) target_title: String,
    pub(super) user_label: String,
    pub(super) user_id: UserId,
    pub(super) user_url: String,
    pub(super) reason: String,
    pub(super) reason_full: String,
    pub(super) chain: Vec<ChainPill>,
    pub(super) chain_overflow: String,
    pub(super) eval_count: i64,
    pub(super) detail_url: String,
    pub(super) trace_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct FindingRow {
    pub(super) created_at: String,
    pub(super) category: String,
    pub(super) scanner: String,
    pub(super) severity: String,
    pub(super) severity_tone: &'static str,
    pub(super) direction: &'static str,
    pub(super) outcome: &'static str,
    pub(super) outcome_tone: &'static str,
    pub(super) excerpt: String,
    pub(super) model: String,
    pub(super) request_url: String,
    pub(super) request_label: String,
}

#[derive(Debug, Serialize)]
pub(super) struct HookRow {
    pub(super) created_at: String,
    pub(super) kind: String,
    pub(super) tool_name: String,
    pub(super) plugin: String,
    pub(super) user_id: UserId,
    pub(super) status: String,
    pub(super) tone: &'static str,
}

pub(super) use crate::types::governance_labels::{plane_of, target_label};

pub(super) const fn decision_tone(decision: &str) -> &'static str {
    match decision.as_bytes() {
        b"deny" => "err",
        b"warn" => "warn",
        b"allow" => "ok",
        _ => "muted",
    }
}

const fn severity_tone(severity: &str) -> &'static str {
    match severity.as_bytes() {
        b"critical" | b"high" => "err",
        b"medium" => "warn",
        _ => "muted",
    }
}

fn dash(value: Option<&str>) -> String {
    value
        .filter(|v| !v.is_empty())
        .map_or_else(|| "\u{2014}".to_owned(), ToOwned::to_owned)
}

const CHAIN_PILLS: usize = 6;

fn pills(chain: &[ChainEvaluation]) -> Vec<ChainPill> {
    let mut out: Vec<ChainPill> = Vec::new();
    let mut last: Option<(&str, &str)> = None;
    for step in chain {
        if last == Some((step.policy.as_str(), step.decision.as_str())) {
            continue;
        }
        last = Some((step.policy.as_str(), step.decision.as_str()));
        let plane = plane_of(&step.policy);
        let title = if step.reason.is_empty() {
            format!("{} · {} · {plane}", step.policy, step.decision)
        } else {
            format!(
                "{} · {} · {plane} — {}",
                step.policy, step.decision, step.reason
            )
        };
        out.push(ChainPill {
            policy: step.policy.clone(),
            tone: decision_tone(&step.decision),
            title,
            href: format!(
                "/admin/governance/decisions/{}",
                urlencoding::encode(&step.id)
            ),
        });
    }
    out
}

fn target_title(chain: &[ChainEvaluation]) -> String {
    let mut seen: Vec<String> = Vec::new();
    for step in chain {
        let label = target_label(&step.tool_name, step.entity_type.as_deref());
        if !seen.contains(&label) {
            seen.push(label);
        }
    }
    seen.join(" · ")
}

pub(super) fn decision_rows(rows: &[DecisionCallRow]) -> Vec<DecisionRow> {
    rows.iter()
        .map(|r| {
            let all = pills(&r.chain.0);
            let shown: Vec<ChainPill> = all.into_iter().take(CHAIN_PILLS).collect();
            let hidden = usize::try_from(r.eval_count)
                .unwrap_or(shown.len())
                .saturating_sub(shown.len());
            DecisionRow {
                created_at: local_time(r.started_at),
                span: span_of(r),
                decision: r.worst_decision.clone(),
                tone: decision_tone(&r.worst_decision),
                attention: r.deny_count + r.warn_count > 0,
                target: target_label(&r.worst_tool, r.worst_entity_type.as_deref()),
                target_title: target_title(&r.chain.0),
                user_label: r.user_label.clone(),
                user_id: r.user_id.clone(),
                user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
                reason: truncate_ellipsis(&r.worst_reason, REASON_CHARS),
                reason_full: r.worst_reason.clone(),
                chain: shown,
                chain_overflow: if hidden > 0 {
                    format!("+{hidden}")
                } else {
                    String::new()
                },
                eval_count: r.eval_count,
                detail_url: format!(
                    "/admin/governance/decisions/{}",
                    urlencoding::encode(&r.worst_id)
                ),
                trace_url: r
                    .trace_id
                    .as_deref()
                    .filter(|t| !t.is_empty())
                    .map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
            }
        })
        .collect()
}

fn span_of(row: &DecisionCallRow) -> String {
    let ms = (row.ended_at - row.started_at).num_milliseconds();
    if ms < 1000 {
        return String::new();
    }
    let seconds = crate::numeric::to_f64(ms) / 1000.0;
    format!("+{seconds:.1}s")
}

pub(super) fn finding_rows(rows: &[SafetyFindingLogRow]) -> Vec<FindingRow> {
    rows.iter()
        .map(|r| FindingRow {
            created_at: local_time(r.created_at),
            category: r.category.clone(),
            scanner: r.scanner.clone(),
            severity: r.severity.clone(),
            severity_tone: severity_tone(&r.severity),
            direction: if r.phase == "request_history" {
                "inbound history"
            } else if r.phase == "request" {
                "inbound"
            } else {
                "outbound"
            },
            outcome: if r.blocked { "blocked" } else { "audited" },
            outcome_tone: if r.blocked { "err" } else { "muted" },
            excerpt: truncate_ellipsis(r.excerpt.as_deref().unwrap_or("\u{2014}"), REASON_CHARS),
            model: dash(r.model.as_deref()),
            request_url: format!("/admin/requests/{}", urlencoding::encode(&r.ai_request_id)),
            request_label: short(&r.ai_request_id),
        })
        .collect()
}

fn short(id: &str) -> String {
    if id.is_empty() {
        return "\u{2014}".to_owned();
    }
    crate::handlers::ssr::format::short_id(id)
}

pub(super) fn hook_rows(events: &[RecentHookEvent]) -> Vec<HookRow> {
    events
        .iter()
        .map(|e| {
            let status = e.status.clone().unwrap_or_else(|| "recorded".to_owned());
            HookRow {
                created_at: local_time(e.created_at),
                kind: e.kind.clone(),
                tool_name: dash(e.tool_name.as_deref()),
                plugin: dash(e.plugin_id.as_ref().map(PluginId::as_str)),
                user_id: e.user_id.clone(),
                tone: decision_tone(&status),
                status,
            }
        })
        .collect()
}

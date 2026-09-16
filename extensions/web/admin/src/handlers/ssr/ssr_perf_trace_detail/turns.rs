//! Folds the flat span list into the rows the trace page reads.
//!
//! A gateway call leaves one `ai_requests` row and, on the same `trace_id`,
//! every governance decision the gateway took for it: two access checks on
//! the route and one policy chain on the prompt. Rendered flat that is four
//! rows per turn, three of them zero-width; here they become one row with a
//! checks summary, and the decisions open under it. A decision with no
//! request on its trace (a hook-side tool call, a pre-flight deny) and every
//! plugin event keep a row of their own.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::format::{
    client_label, format_cost, format_duration_ms, local_time, short_id, short_num,
};
use crate::repositories::traces::{PolicyStage, Span, SpanKind, SpanStatus};

// Why: a pause longer than this between rows is shown as a gap, so the
// bursts a person's session naturally falls into read as bursts.
const GAP_THRESHOLD_MS: i64 = 30_000;

#[derive(Debug, Serialize)]
pub(super) struct TraceTurnView {
    pub(super) index: usize,
    pub(super) id: String,
    pub(super) detail_id: String,
    pub(super) kind: &'static str,
    pub(super) is_request: bool,
    pub(super) name: String,
    pub(super) sub: String,
    pub(super) started_at_local: String,
    pub(super) offset_label: String,
    pub(super) gap_label: Option<String>,
    pub(super) status: &'static str,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) latency_display: String,
    pub(super) tokens_display: String,
    pub(super) cache_display: Option<String>,
    pub(super) cost_display: String,
    pub(super) context_id_short: Option<String>,
    pub(super) context_url: Option<String>,
    pub(super) url: Option<String>,
    pub(super) error_message: Option<String>,
    pub(super) checks: ChecksView,
}

#[derive(Debug, Serialize)]
pub(super) struct ChecksView {
    pub(super) count: usize,
    pub(super) label: String,
    pub(super) tone: &'static str,
    pub(super) items: Vec<CheckView>,
}

#[derive(Debug, Serialize)]
pub(super) struct CheckView {
    pub(super) id: String,
    pub(super) url: String,
    pub(super) policy: String,
    pub(super) subject: String,
    pub(super) decision: String,
    pub(super) tone: &'static str,
    pub(super) reason: Option<String>,
    pub(super) offset_label: String,
    pub(super) stages: Vec<StageView>,
}

#[derive(Debug, Serialize)]
pub(super) struct StageView {
    pub(super) policy_id: String,
    pub(super) result: String,
    pub(super) tone: &'static str,
    pub(super) detail: String,
    pub(super) duration_display: String,
}

pub(super) fn build_turns(spans: &[Span]) -> Vec<TraceTurnView> {
    let start = spans.iter().map(|s| s.started_at).min();
    let offset_of = |s: &Span| start.map_or(0, |a| (s.started_at - a).num_milliseconds().max(0));

    // Why: the request that owns a trace id, so its decisions fold under it.
    let mut owners: BTreeMap<&str, &str> = BTreeMap::new();
    for s in spans {
        if let (SpanKind::Model, Some(t)) = (s.kind, s.trace_id.as_deref()) {
            owners.entry(t).or_insert(s.id.as_str());
        }
    }
    let mut folded: BTreeMap<&str, Vec<&Span>> = BTreeMap::new();
    let mut rows: Vec<&Span> = Vec::new();
    for s in spans {
        let owner = match (s.kind, s.trace_id.as_deref()) {
            (SpanKind::Governance, Some(t)) => owners.get(t).copied(),
            _ => None,
        };
        match owner {
            Some(id) => folded.entry(id).or_default().push(s),
            None => rows.push(s),
        }
    }

    let mut previous_end: Option<chrono::DateTime<chrono::Utc>> = None;
    rows.iter()
        .enumerate()
        .map(|(n, s)| {
            let checks = folded.get(s.id.as_str()).map_or(&[][..], Vec::as_slice);
            let gap_label = previous_end
                .map(|e| (s.started_at - e).num_milliseconds())
                .filter(|g| *g >= GAP_THRESHOLD_MS)
                .map(|g| format!("{} quiet", format_duration_ms(g)));
            previous_end = Some(previous_end.map_or(s.ended_at, |e| e.max(s.ended_at)));
            build_turn(n + 1, s, checks, gap_label, &offset_of)
        })
        .collect()
}

fn build_turn(
    index: usize,
    s: &Span,
    checks: &[&Span],
    gap_label: Option<String>,
    offset_of: &dyn Fn(&Span) -> i64,
) -> TraceTurnView {
    let is_request = matches!(s.kind, SpanKind::Model);
    let (status_label, status_tone) = status_badge(s.status);
    let context = s.context_id.as_ref().filter(|c| !c.as_str().is_empty());
    let url = match s.kind {
        SpanKind::Model => Some(format!("/admin/requests/{}", urlencoding::encode(&s.id))),
        SpanKind::Governance => Some(format!(
            "/admin/governance/decisions/{}",
            urlencoding::encode(&s.id)
        )),
        _ => None,
    };
    // Why: under a model the second line names the client; under a decision
    // or hook event it names the policy or event type.
    let sub = match s.kind {
        SpanKind::Model => s
            .client_kind
            .as_deref()
            .map(client_label)
            .unwrap_or_default(),
        _ => s.policy.clone().unwrap_or_default(),
    };
    TraceTurnView {
        index,
        id: s.id.clone(),
        detail_id: format!("turn-{index}"),
        kind: s.kind.as_str(),
        is_request,
        name: s.name.clone(),
        sub,
        started_at_local: local_time(s.started_at),
        offset_label: offset_label(offset_of(s)),
        gap_label,
        status: s.status.as_str(),
        status_label,
        status_tone,
        latency_display: s
            .latency_ms
            .map_or_else(|| "—".to_owned(), format_duration_ms),
        tokens_display: tokens(s.input_tokens, s.output_tokens),
        cache_display: s
            .cache_read_tokens
            .filter(|c| *c > 0)
            .map(|c| format!("{} cached", short_num(c))),
        cost_display: s
            .cost_microdollars
            .map_or_else(|| "—".to_owned(), format_cost),
        context_id_short: context.map(|c| short_id(c.as_str())),
        context_url: context.map(context_detail_url),
        url,
        error_message: s.error_message.clone().or_else(|| match s.kind {
            SpanKind::Governance => s.reason.clone(),
            _ => None,
        }),
        checks: build_checks(s, checks, offset_of),
    }
}

// Why: a standalone decision row is its own check, so the expanded panel
// shows its stages exactly as it would under a request.
fn build_checks(s: &Span, folded: &[&Span], offset_of: &dyn Fn(&Span) -> i64) -> ChecksView {
    let sources: Vec<&Span> = if matches!(s.kind, SpanKind::Governance) {
        vec![s]
    } else {
        folded.to_vec()
    };
    let items: Vec<CheckView> = sources
        .iter()
        .map(|d| CheckView {
            id: d.id.clone(),
            url: format!("/admin/governance/decisions/{}", urlencoding::encode(&d.id)),
            policy: d.policy.clone().unwrap_or_default(),
            subject: d.name.clone(),
            decision: d.decision.clone().unwrap_or_else(|| "allow".to_owned()),
            tone: status_badge(d.status).1,
            reason: d.reason.clone(),
            offset_label: offset_label(offset_of(d)),
            stages: d.stages.iter().map(stage_view).collect(),
        })
        .collect();
    let denies = sources
        .iter()
        .filter(|d| matches!(d.status, SpanStatus::Deny))
        .count();
    let warns = sources
        .iter()
        .filter(|d| matches!(d.status, SpanStatus::Warn))
        .count();
    let stage_count: usize = items.iter().map(|c| c.stages.len()).sum();
    let (label, tone) = if items.is_empty() {
        ("no checks".to_owned(), "muted")
    } else if denies > 0 {
        (format!("{denies} denied"), "err")
    } else if warns > 0 {
        (format!("{warns} warned"), "warn")
    } else if stage_count > 0 {
        (
            format!("{} checks · {stage_count} policies", items.len()),
            "ok",
        )
    } else {
        (format!("{} checks", items.len()), "ok")
    };
    ChecksView {
        count: items.len(),
        label,
        tone,
        items,
    }
}

fn stage_view(st: &PolicyStage) -> StageView {
    let tone = match st.result.as_str() {
        "pass" => "ok",
        "deny" | "fail" | "block" => "err",
        "warn" => "warn",
        _ => "muted",
    };
    StageView {
        policy_id: st.policy_id.clone(),
        result: st.result.clone(),
        tone,
        detail: st.detail.clone(),
        duration_display: if st.duration_ms >= 1.0 {
            format!("{:.0} ms", st.duration_ms)
        } else {
            "<1 ms".to_owned()
        },
    }
}

const fn status_badge(status: SpanStatus) -> (&'static str, &'static str) {
    match status {
        SpanStatus::Ok => ("OK", "ok"),
        SpanStatus::Deny => ("DENIED", "err"),
        SpanStatus::Warn => ("WARN", "warn"),
        SpanStatus::Error => ("FAILED", "err"),
        SpanStatus::Pending => ("PENDING", "muted"),
        SpanStatus::Rejected => ("REJECTED", "warn"),
    }
}

fn tokens(input: Option<i64>, output: Option<i64>) -> String {
    match (input, output) {
        (Some(i), Some(o)) => format!("{} → {}", short_num(i), short_num(o)),
        (Some(i), None) => format!("{} →", short_num(i)),
        _ => "—".to_owned(),
    }
}

fn offset_label(ms: i64) -> String {
    format!("+{}", format_duration_ms(ms))
}

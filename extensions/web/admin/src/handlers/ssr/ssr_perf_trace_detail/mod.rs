//! `/admin/traces/{id}` — what one session did, turn by turn.
//!
//! `id` may be a `session_id` or a `trace_id`. The handler resolves it to a
//! session id, fetches every span, and renders a timeline strip (see
//! [`timeline`]) over a turns table (see [`turns`]): one row per gateway call
//! with its governance checks folded under it, one row per tool event or
//! standalone decision, and a gap marker wherever the person paused. Every
//! row links to the request or decision audit page.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::{
    client_label, format_cost, format_duration_ms, local_time, short_id, short_num,
};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::scope::may_view;
use crate::repositories::traces::resolve::find_trace_owner;
use crate::repositories::traces::{
    Span, SpanKind, SpanStatus, list_trace_spans, resolve_trace_session,
};

mod timeline;
mod turns;

use crate::handlers::ssr::page::Page;
use timeline::{TimelineView, build_timeline};
use turns::{TraceTurnView, build_turns};

#[derive(Debug, Serialize)]
struct TraceDetailContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    summary: Summary,
    timeline: TimelineView,
    turns: Vec<TraceTurnView>,
    turn_count: usize,
    back_url: &'static str,
}

#[derive(Debug, Serialize)]
struct Summary {
    session_id: SessionId,
    session_id_short: String,
    session_url: String,
    duration_ms: i64,
    started_at: Option<String>,
    started_at_local: Option<String>,
    duration_display: String,
    user_label: String,
    user_url: String,
    client_label: Option<String>,
    span_count: usize,
    request_count: usize,
    requests_note: String,
    failed_count: usize,
    cost_display: String,
    tokens_display: String,
    tokens_note: String,
    check_count: usize,
    checks_note: String,
    deny_count: usize,
    event_count: usize,
    events_note: String,
}

fn no_visible_trace() -> AdminError {
    AdminError::NotFound("No spans found for that session or trace id.".to_owned())
}

pub(crate) async fn perf_trace_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    let Some(session_id) = resolve_trace_session(&pool, &id).await? else {
        return Err(no_visible_trace().into());
    };

    let owner = find_trace_owner(&pool, &session_id).await?;
    if !may_view(&pool, &shell.user, owner.as_ref()).await? {
        return Err(no_visible_trace().into());
    }

    let spans = list_trace_spans(&pool, &session_id).await?;

    if spans.is_empty() {
        return Err(no_visible_trace().into());
    }

    let summary = build_summary(&session_id, &spans);
    let turns = build_turns(&spans);

    let ctx = TraceDetailContext {
        page: "trace-detail",
        title: format!("Trace · {}", short_id(session_id.as_str())),
        breadcrumbs: breadcrumbs(&session_id),
        timeline: build_timeline(&spans, summary.duration_ms),
        turn_count: turns.len(),
        summary,
        turns,
        back_url: "/admin/traces",
    };

    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "perf-trace-detail",
        &ctx,
        &shell.user,
        &shell.marketplace,
    ))
}

fn breadcrumbs(session_id: &SessionId) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Traces", "/admin/traces"),
        BreadcrumbView::current(short_id(session_id.as_str())),
    ]
}

struct Tally {
    requests: usize,
    failed: usize,
    rejected: usize,
    checks: usize,
    denied: usize,
    warned: usize,
    input: i64,
    output: i64,
    cached: i64,
    cost: i64,
}

fn tally(spans: &[Span]) -> Tally {
    let requests = spans.iter().filter(|s| matches!(s.kind, SpanKind::Model));
    let checks = spans
        .iter()
        .filter(|s| matches!(s.kind, SpanKind::Governance));
    Tally {
        requests: requests.clone().count(),
        failed: requests
            .clone()
            .filter(|s| matches!(s.status, SpanStatus::Error))
            .count(),
        rejected: requests
            .clone()
            .filter(|s| matches!(s.status, SpanStatus::Rejected))
            .count(),
        checks: checks.clone().count(),
        denied: checks
            .clone()
            .filter(|s| matches!(s.status, SpanStatus::Deny))
            .count(),
        warned: checks
            .filter(|s| matches!(s.status, SpanStatus::Warn))
            .count(),
        input: requests.clone().filter_map(|s| s.input_tokens).sum(),
        output: requests.clone().filter_map(|s| s.output_tokens).sum(),
        cached: requests.clone().filter_map(|s| s.cache_read_tokens).sum(),
        cost: requests.filter_map(|s| s.cost_microdollars).sum(),
    }
}

fn build_summary(session_id: &SessionId, spans: &[Span]) -> Summary {
    let started = spans.iter().map(|s| s.started_at).min();
    let ended = spans.iter().map(|s| s.ended_at).max();
    let total_ms = match (started, ended) {
        (Some(a), Some(b)) => (b - a).num_milliseconds().max(0),
        _ => 0,
    };
    let t = tally(spans);
    let events: Vec<&Span> = spans
        .iter()
        .filter(|s| matches!(s.kind, SpanKind::Tool | SpanKind::Spawn))
        .collect();

    let client = spans
        .iter()
        .filter(|s| matches!(s.kind, SpanKind::Model))
        .find_map(|s| s.client_kind.as_deref())
        .map(client_label);
    let person = spans.first().map_or_else(
        || ("unknown".to_owned(), String::new()),
        |s| {
            (
                s.identity_label
                    .clone()
                    .unwrap_or_else(|| s.user_id.to_string()),
                format!("/admin/users/{}", urlencoding::encode(s.user_id.as_str())),
            )
        },
    );

    Summary {
        session_url: session_detail_url(session_id),
        session_id: session_id.clone(),
        session_id_short: short_id(session_id.as_str()),
        duration_ms: total_ms,
        started_at: started.map(|t| t.to_rfc3339()),
        started_at_local: started.map(local_time),
        duration_display: format_duration_ms(total_ms),
        user_label: person.0,
        user_url: person.1,
        client_label: client,
        span_count: spans.len(),
        request_count: t.requests,
        requests_note: requests_note(t.requests, t.failed, t.rejected),
        failed_count: t.failed + t.rejected,
        cost_display: format_cost(t.cost),
        tokens_display: if t.input + t.output > 0 {
            format!("{} → {}", short_num(t.input), short_num(t.output))
        } else {
            "—".to_owned()
        },
        tokens_note: if t.cached > 0 {
            format!("in → out · {} read from cache", short_num(t.cached))
        } else {
            "in → out".to_owned()
        },
        check_count: t.checks,
        checks_note: checks_note(t.checks, t.denied, t.warned),
        deny_count: t.denied,
        event_count: events.len(),
        events_note: events_note(&events),
    }
}

fn requests_note(total: usize, failed: usize, rejected: usize) -> String {
    match (failed, rejected) {
        (0, 0) if total > 0 => "all completed".to_owned(),
        (0, 0) => "no gateway calls".to_owned(),
        (f, 0) => format!("{f} failed"),
        (0, r) => format!("{r} rejected before a provider"),
        (f, r) => format!("{f} failed · {r} rejected"),
    }
}

fn checks_note(total: usize, denied: usize, warned: usize) -> String {
    match (denied, warned) {
        (0, 0) if total > 0 => "every check allowed".to_owned(),
        (0, 0) => "no governance decisions".to_owned(),
        (d, 0) => format!("{d} denied"),
        (0, w) => format!("{w} warned"),
        (d, w) => format!("{d} denied · {w} warned"),
    }
}

fn events_note(events: &[&Span]) -> String {
    if events.is_empty() {
        return "no hook events".to_owned();
    }
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for e in events {
        *counts
            .entry(e.policy.as_deref().unwrap_or("event"))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map_or_else(String::new, |(name, n)| format!("{n}× {name}"))
}

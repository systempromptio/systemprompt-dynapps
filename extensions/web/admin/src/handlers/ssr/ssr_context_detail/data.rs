//! View-model builders for the single-context detail page: map repository rows
//! into the typed template-context structs in `context`.
//!
//! The chronological transcript itself is built by
//! `handlers::ssr::transcript_view`, which the owner-facing conversation page
//! under `/admin/history` shares — this page asks it for the unredacted,
//! unstripped view an admin is entitled to.

use crate::repositories::analytics::context_detail::{
    ContextHeader, ContextKpis, ContextMessageRow, ContextRequestRow, ContextToolCallRow,
};

use crate::types::conversation_analytics::SessionEntityLink;

use super::context::{
    BreadcrumbView, ContextDetailPageContext, ContextRequestRowView, EntityLinkView, HeaderView,
    KpisView, TranscriptEntryView,
};
use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url, trace_detail_url};
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::transcript_view::{
    TranscriptOptions, build_transcript, format_latency, short_id,
};

pub(super) const fn default_kpis() -> ContextKpis {
    ContextKpis {
        request_count: 0,
        trace_count: 0,
        error_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_microdollars: 0,
        first_request_at: None,
        last_request_at: None,
        model: None,
    }
}

// Why: Everything the page renders beside its header, gathered so the builder
// stays inside clippy's argument cap.
pub(super) struct DetailInputs<'a> {
    pub kpis: &'a ContextKpis,
    pub requests: &'a [ContextRequestRow],
    pub messages: &'a [ContextMessageRow],
    pub tool_calls: &'a [ContextToolCallRow],
    pub entity_links: &'a [SessionEntityLink],
}

pub(super) fn build_detail_data(
    header: &ContextHeader,
    inputs: &DetailInputs<'_>,
) -> ContextDetailPageContext {
    let DetailInputs {
        kpis,
        requests,
        messages,
        tool_calls,
        entity_links,
    } = *inputs;
    let entity_link_views = entity_link_views(entity_links);
    let transcript: Vec<TranscriptEntryView> =
        build_transcript(messages, tool_calls, requests, TranscriptOptions::default()).entries;
    let back_url = header
        .session_id
        .as_ref()
        .map_or_else(|| "/admin/overview/pulse".to_owned(), session_detail_url);
    let back_label = header
        .session_id
        .as_ref()
        .map_or_else(|| "Live ticker".to_owned(), |_| "Session".to_owned());
    ContextDetailPageContext {
        page: "context-detail",
        title: format!("Context · {}", short_id(header.context_id.as_str())),
        header: header_view(header),
        kpis: kpis_view(kpis),
        has_transcript: !messages.is_empty() || !tool_calls.is_empty(),
        transcript,
        has_requests: !requests.is_empty(),
        requests: requests.iter().map(request_view).collect(),
        back_url,
        back_label,
        breadcrumbs: breadcrumbs(header),
        has_entity_links: !entity_link_views.is_empty(),
        entity_link_count: entity_link_views.len(),
        entity_links: entity_link_views,
    }
}

fn breadcrumbs(h: &ContextHeader) -> Vec<BreadcrumbView> {
    let mut crumbs = vec![BreadcrumbView::link("Contexts", "/admin/contexts")];
    if let Some(session) = h.session_id.as_ref() {
        crumbs.push(BreadcrumbView::link(
            format!("Session {}", short_id(session.as_str())),
            session_detail_url(session),
        ));
    }
    crumbs.push(BreadcrumbView::current(short_id(h.context_id.as_str())));
    crumbs
}

// Why: the share is measured against the busiest entity, not the total —
// a hundred touches of one file and one of another should read as a full bar
// beside an empty one, which a share-of-total would flatten.
fn entity_link_views(links: &[SessionEntityLink]) -> Vec<EntityLinkView> {
    let max = links.iter().map(|l| l.usage_count).max().unwrap_or(0);
    links
        .iter()
        .map(|l| EntityLinkView {
            entity_type: l.entity_type.clone(),
            entity_name: l.entity_name.clone(),
            usage_count: l.usage_count,
            share_pct: if max > 0 {
                (f64::from(l.usage_count) / f64::from(max) * 100.0).round()
            } else {
                0.0
            },
        })
        .collect()
}

fn header_view(h: &ContextHeader) -> HeaderView {
    HeaderView {
        context_id: h.context_id.clone(),
        context_id_short: short_id(h.context_id.as_str()),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/user?id={}", urlencoding::encode(u.as_str()))),
        display_name: h.display_name.clone(),
        session_id: h.session_id.clone(),
        session_url: h.session_id.as_ref().map(session_detail_url),
        name: h.name.clone(),
        created_at: h.created_at.map(|t| t.to_rfc3339()),
        created_at_local: h.created_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        updated_at_local: h.updated_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
    }
}

fn kpis_view(k: &ContextKpis) -> KpisView {
    KpisView {
        request_count: k.request_count,
        trace_count: k.trace_count,
        error_count: k.error_count,
        total_input_tokens: k.total_input_tokens,
        total_output_tokens: k.total_output_tokens,
        total_tokens: k.total_input_tokens + k.total_output_tokens,
        total_cost_microdollars: k.total_cost_microdollars,
        total_cost_display: format_cost(k.total_cost_microdollars),
        model: k.model.clone(),
        first_request_at_local: k.first_request_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
        last_request_at_local: k.last_request_at.map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        }),
    }
}

fn request_view(r: &ContextRequestRow) -> ContextRequestRowView {
    ContextRequestRowView {
        id: r.id.clone(),
        id_short: short_id(r.id.as_str()),
        request_url: request_detail_url(&r.id),
        trace_id: r.trace_id.clone(),
        trace_id_short: r.trace_id.as_ref().map(|t| short_id(t.as_str())),
        trace_url: r.trace_id.as_ref().map(trace_detail_url),
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        latency_display: format_latency(r.latency_ms),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    }
}

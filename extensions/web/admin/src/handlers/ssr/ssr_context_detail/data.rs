//! Loads the conversation reader's rows and maps them into the typed
//! template-context structs in `context`.
//!
//! Loading runs in two phases because message and tool-call bodies are fetched
//! by request id: the request rollup names the handful of requests the
//! transcript is built from. Tool calls are the one load that fails the page
//! instead of degrading to empty — no KPI counts them, so an empty list would
//! silently redact every tool invocation from the evidence.
//!
//! The transcript itself is built by `handlers::ssr::transcript_view`, which
//! the owner-facing page under `/admin/history` shares — this page asks it for
//! the unredacted, unstripped view an admin is entitled to.

use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::repositories::analytics::context_detail::{
    ContextHeader, ContextKpis, ContextMessageRow, ContextRequestRow, get_context_kpis,
    list_context_requests, list_messages_for_requests,
};
use crate::repositories::analytics::context_tool_calls::{
    ContextToolCallRow, list_tool_calls_for_requests,
};
use crate::repositories::dashboard::conversation_analytics::list_session_entity_links;

use crate::types::conversation_analytics::SessionEntityLink;

use super::context::{
    BreadcrumbView, ContextDetailPageContext, ContextRequestRowView, EntityLinkView, HeaderView,
    TabLinkView,
};
use crate::handlers::ssr::conversation_header::{
    resolve_title, stats_view, status_badge, timeline_display,
};
use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url, trace_detail_url};
use crate::handlers::ssr::format::{format_cost, local_time};
use crate::handlers::ssr::transcript_view::{
    TranscriptOptions, build_conversation, format_latency, short_id, transcript_request_ids,
};

pub(super) const TAB_CONVERSATION: &str = "conversation";
pub(super) const TAB_REQUESTS: &str = "requests";
pub(super) const TAB_TOUCHED: &str = "touched";

pub(super) fn resolve_tab(requested: Option<&str>) -> &'static str {
    match requested {
        Some(TAB_REQUESTS) => TAB_REQUESTS,
        Some(TAB_TOUCHED) => TAB_TOUCHED,
        _ => TAB_CONVERSATION,
    }
}

const fn default_kpis() -> ContextKpis {
    ContextKpis {
        request_count: 0,
        trace_count: 0,
        error_count: 0,
        total_input_tokens: 0,
        total_cache_read_tokens: 0,
        total_cache_creation_tokens: 0,
        total_output_tokens: 0,
        total_cost_microdollars: 0,
        first_request_at: None,
        last_request_at: None,
        model: None,
        turn_count: 0,
        side_call_count: 0,
        side_call_cost_microdollars: 0,
        tool_call_count: 0,
        models: Vec::new(),
    }
}

pub(super) struct TranscriptRead {
    kpis: ContextKpis,
    requests: Vec<ContextRequestRow>,
    messages: Vec<ContextMessageRow>,
    tool_calls: Vec<ContextToolCallRow>,
    entity_links: Vec<SessionEntityLink>,
}

pub(super) async fn load_transcript(
    pool: &PgPool,
    header: &ContextHeader,
    context_id: &ContextId,
) -> Result<TranscriptRead, sqlx::Error> {
    let (kpis, requests) = load_request_rollup(pool, context_id).await;
    let ids = transcript_request_ids(&requests);
    let (messages_res, tool_calls_res) = tokio::join!(
        list_messages_for_requests(pool, &ids.messages),
        list_tool_calls_for_requests(pool, &ids.tool_calls),
    );
    let messages = messages_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_messages_for_requests failed");
        Vec::new()
    });
    let tool_calls = tool_calls_res?;
    let entity_links = load_session_entity_links(pool, header).await;
    Ok(TranscriptRead {
        kpis,
        requests,
        messages,
        tool_calls,
        entity_links,
    })
}

async fn load_request_rollup(
    pool: &PgPool,
    context_id: &ContextId,
) -> (ContextKpis, Vec<ContextRequestRow>) {
    let (kpis_res, requests_res) = tokio::join!(
        get_context_kpis(pool, context_id),
        list_context_requests(pool, context_id),
    );
    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_context_kpis failed");
        default_kpis()
    });
    let requests = requests_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_context_requests failed");
        Vec::new()
    });
    (kpis, requests)
}

async fn load_session_entity_links(
    pool: &PgPool,
    header: &ContextHeader,
) -> Vec<SessionEntityLink> {
    let Some(session_id) = header.session_id.as_ref() else {
        return Vec::new();
    };
    list_session_entity_links(pool, session_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_session_entity_links failed");
            Vec::new()
        })
}

pub(super) fn build_detail_data(
    header: &ContextHeader,
    read: &TranscriptRead,
    active_tab: &'static str,
) -> ContextDetailPageContext {
    let TranscriptRead {
        kpis,
        requests,
        messages,
        tool_calls,
        entity_links,
    } = read;
    let entity_link_views = entity_link_views(entity_links);
    let conversation =
        build_conversation(messages, tool_calls, requests, TranscriptOptions::default());
    let title = resolve_title(header, &conversation);
    let back_url = header
        .session_id
        .as_ref()
        .map_or_else(|| "/admin/conversations".to_owned(), session_detail_url);
    let back_label = header
        .session_id
        .as_ref()
        .map_or_else(|| "Conversations".to_owned(), |_| "Session".to_owned());
    ContextDetailPageContext {
        page: "context-detail",
        export: crate::export::ExportView::conversation(&header.context_id, true),
        tabs: tab_links(header, active_tab, requests.len(), entity_link_views.len()),
        show_conversation: active_tab == TAB_CONVERSATION,
        show_requests: active_tab == TAB_REQUESTS,
        show_touched: active_tab == TAB_TOUCHED,
        header: header_view(header, kpis),
        stats: stats_view(kpis),
        status_badge: status_badge(header.hook_status.as_deref(), kpis.error_count),
        conversation,
        has_requests: !requests.is_empty(),
        request_count: requests.len(),
        requests: requests.iter().map(request_view).collect(),
        back_url,
        back_label,
        breadcrumbs: breadcrumbs(header, &title),
        has_entity_links: !entity_link_views.is_empty(),
        entity_link_count: entity_link_views.len(),
        entity_links: entity_link_views,
        title,
    }
}

fn tab_links(
    h: &ContextHeader,
    active: &str,
    request_count: usize,
    touched_count: usize,
) -> Vec<TabLinkView> {
    let base = format!(
        "/admin/contexts/{}",
        urlencoding::encode(h.context_id.as_str())
    );
    let tab = |slug: &'static str, label: &'static str, count: Option<usize>| TabLinkView {
        slug,
        label,
        href: format!("{base}?tab={slug}"),
        is_active: slug == active,
        count: count.and_then(|c| i64::try_from(c).ok()),
    };
    vec![
        tab(TAB_CONVERSATION, "Conversation", None),
        tab(TAB_REQUESTS, "Requests", Some(request_count)),
        tab(TAB_TOUCHED, "Touched", Some(touched_count)),
    ]
}

fn breadcrumbs(h: &ContextHeader, title: &str) -> Vec<BreadcrumbView> {
    let mut crumbs = vec![BreadcrumbView::link(
        "Conversations",
        "/admin/conversations",
    )];
    if let Some(session) = h.session_id.as_ref() {
        crumbs.push(BreadcrumbView::link(
            format!("Session {}", short_id(session.as_str())),
            session_detail_url(session),
        ));
    }
    crumbs.push(BreadcrumbView::current(title));
    crumbs
}

fn entity_link_views(links: &[SessionEntityLink]) -> Vec<EntityLinkView> {
    let busiest = links.iter().map(|l| l.usage_count).max().unwrap_or(0);
    links
        .iter()
        .map(|l| EntityLinkView {
            entity_type: l.entity_type.clone(),
            entity_name: l.entity_name.clone(),
            usage_count: l.usage_count,
            share_pct: share_of_busiest_pct(l.usage_count, busiest),
        })
        .collect()
}

fn share_of_busiest_pct(usage_count: i32, busiest: i32) -> f64 {
    if busiest > 0 {
        (f64::from(usage_count) / f64::from(busiest) * 100.0).round()
    } else {
        0.0
    }
}

fn header_view(h: &ContextHeader, k: &ContextKpis) -> HeaderView {
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
        client_session_id: h.client_session_id.clone(),
        hooks_session_url: h
            .client_session_id
            .as_ref()
            .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        timeline: timeline_display(k.first_request_at, k.last_request_at),
        first_at: k.first_request_at.map(local_time),
        last_at: k.last_request_at.map(local_time),
    }
}

const fn kind_tone(kind: &str) -> &'static str {
    match kind.as_bytes() {
        b"probe" => "muted",
        b"utility" => "info",
        _ => "accent",
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
        kind_tone: kind_tone(&r.effective_kind),
        kind: r.effective_kind.clone(),
        message_count: r.message_count,
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        input_tokens: r.input_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_creation_tokens: r.cache_creation_tokens,
        output_tokens: r.output_tokens,
        max_tokens: r.max_tokens,
        latency_ms: r.latency_ms,
        cost_microdollars: r.cost_microdollars,
        created_at: r.created_at.to_rfc3339(),
        latency_display: format_latency(r.latency_ms),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: local_time(r.created_at),
    }
}

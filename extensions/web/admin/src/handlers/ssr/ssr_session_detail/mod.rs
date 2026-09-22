//! `/admin/sessions/{session_id}` — single-session detail page.
//!
//! Renders the header, KPI strip, and three linked tables (contexts, traces,
//! requests) for one `session_id`, mirroring `analytics sessions stats` plus
//! `analytics conversations list` with cross-links to the other entity pages.

mod context;
mod quality;
mod rows;


use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::format::{
    format_cost, format_duration_ms, format_token_total, local_time, short_id,
};
use crate::handlers::ssr::page::Page;
use crate::repositories::analytics::session_detail::{
    SessionContextRow, SessionHeader, SessionKpis, SessionRequestRow, SessionTraceRow,
    find_session_header, get_session_kpis, list_session_contexts, list_session_requests,
    list_session_traces,
};
use crate::repositories::analytics::session_quality::{SessionRatingRow, list_session_ratings};
use crate::repositories::scope::may_view;

use rows::{context_view, request_view, trace_view};

use context::{BreadcrumbView, SessionDetailPageContext, SessionHeaderView, SessionKpisView};


struct SessionRead {
    kpis: SessionKpis,
    contexts: Vec<SessionContextRow>,
    traces: Vec<SessionTraceRow>,
    requests: Vec<SessionRequestRow>,
    ratings: Vec<SessionRatingRow>,
}

// Why: every section is best-effort once the header has resolved — a panel
// that cannot load renders empty rather than taking the page down.
async fn load_session(pool: &PgPool, session_id: &SessionId) -> SessionRead {
    let (kpis, contexts, traces, requests, ratings) = tokio::join!(
        get_session_kpis(pool, session_id),
        list_session_contexts(pool, session_id),
        list_session_traces(pool, session_id),
        list_session_requests(pool, session_id),
        list_session_ratings(pool, session_id),
    );
    SessionRead {
        kpis: kpis
            .inspect_err(|e| tracing::warn!(error = %e, "get_session_kpis failed"))
            .unwrap_or_default(),
        contexts: contexts
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_contexts failed"))
            .unwrap_or_default(),
        traces: traces
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_traces failed"))
            .unwrap_or_default(),
        requests: requests
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_requests failed"))
            .unwrap_or_default(),
        ratings: ratings
            .inspect_err(|e| tracing::warn!(error = %e, "list_session_ratings failed"))
            .unwrap_or_default(),
    }
}

pub(crate) async fn session_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
) -> AdminHtmlResult<Response> {
    let session_id = SessionId::new(session_id.trim());
    let header = if session_id.as_str().is_empty() {
        None
    } else {
        find_session_header(&pool, &session_id).await?
    };
    // Why: 404, not 403, for an owner outside the caller's view — the same
    // answer as a missing id, so the URL is no oracle for other people's ids.
    let visible = match &header {
        Some(h) => may_view(&pool, &shell.user, h.user_id.as_ref()).await?,
        None => false,
    };
    let Some(header) = header.filter(|_| visible) else {
        return Err(AdminError::NotFound(
            "No AI requests, contexts, or transcript rows match that session id.".to_owned(),
        )
        .into());
    };

    let read = load_session(&pool, &session_id).await;
    let rating_average = quality::rating_average(&read.ratings);
    let rating_views: Vec<_> = read.ratings.iter().map(quality::rating_view).collect();

    let data = SessionDetailPageContext {
        page: "session-detail",
        title: format!("Session · {}", short_id(header.session_id.as_str())),
        breadcrumbs: breadcrumbs(&header),
        header: header_view(&header),
        kpis: kpis_view(&read.kpis, &session_id),
        has_contexts: !read.contexts.is_empty(),
        contexts: read.contexts.iter().map(context_view).collect(),
        has_traces: !read.traces.is_empty(),
        traces: read.traces.iter().map(trace_view).collect(),
        has_requests: !read.requests.is_empty(),
        requests: read.requests.iter().map(request_view).collect(),
        back_url: "/admin/sessions",
        document_export: crate::export::document::DocumentExportView::session(&header.session_id),
        has_ratings: !rating_views.is_empty(),
        rating_count: rating_views.len(),
        rating_average,
        ratings: rating_views,
    };

    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "session-detail",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

fn breadcrumbs(h: &SessionHeader) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Sessions", "/admin/sessions"),
        BreadcrumbView::current(short_id(h.session_id.as_str())),
    ]
}

fn header_view(h: &SessionHeader) -> SessionHeaderView {
    SessionHeaderView {
        session_id: h.session_id.clone(),
        session_id_short: short_id(h.session_id.as_str()),
        user_id: h.user_id.clone(),
        user_url: h
            .user_id
            .as_ref()
            .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
        display_name: h.display_name.clone(),
        groups_display: (!h.groups.is_empty()).then(|| h.groups.join(", ")),
        started_at: h.started_at.map(|t| t.to_rfc3339()),
        started_at_local: h.started_at.map(local_time),
        last_activity_at: h.last_activity_at.map(|t| t.to_rfc3339()),
        last_activity_at_local: h.last_activity_at.map(local_time),
        duration_display: duration_display(h.started_at, h.last_activity_at),
        status: h.status.as_deref().unwrap_or("ended").to_owned(),
        model: h.model.clone(),
        plugin_id: h.plugin_id.clone(),
        ai_title: h.ai_title.clone(),
    }
}

fn kpis_view(k: &SessionKpis, session_id: &SessionId) -> SessionKpisView {
    let wire_input =
        k.total_input_tokens + k.total_cache_read_tokens + k.total_cache_creation_tokens;
    let total = wire_input + k.total_output_tokens;
    SessionKpisView {
        request_count: k.request_count,
        context_count: k.context_count,
        trace_count: k.trace_count,
        error_count: k.error_count,
        total_input_tokens: k.total_input_tokens,
        total_cache_read_tokens: k.total_cache_read_tokens,
        total_cache_creation_tokens: k.total_cache_creation_tokens,
        total_output_tokens: k.total_output_tokens,
        total_tokens: total,
        tokens_display: format_token_total(total),
        tokens_note: tokens_note(wire_input, k.total_cache_read_tokens, k.total_output_tokens),
        total_cost_microdollars: k.total_cost_microdollars,
        total_cost_display: format_cost(k.total_cost_microdollars),
        tool_call_count: k.tool_call_count,
        executed_count: k.executed_count,
        artifact_count: k.artifact_count,
        artifacts_url: format!(
            "/admin/artifacts?session={}&all=1&preset=30d",
            urlencoding::encode(session_id.as_str())
        ),
    }
}

// Why: the cached share is what explains a large input total — see
// `conversation_header::tokens_note`, which the context reader uses for the
// same strip.
fn tokens_note(wire_input: i64, cached: i64, output: i64) -> String {
    let cached_note = if wire_input > 0 && cached > 0 {
        format!(
            " ({} cached, {}%)",
            format_token_total(cached),
            cached.saturating_mul(100) / wire_input
        )
    } else {
        String::new()
    };
    format!(
        "{} in{cached_note} / {} out",
        format_token_total(wire_input),
        format_token_total(output)
    )
}

fn duration_display(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(s), Some(e)) => format_duration_ms((e - s).num_milliseconds().max(0)),
        _ => "—".to_owned(),
    }
}

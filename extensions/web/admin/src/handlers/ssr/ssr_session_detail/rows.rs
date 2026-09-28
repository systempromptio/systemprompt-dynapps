//! Row view-models for the session page's three linked tables: contexts,
//! traces and requests.

use super::context::{SessionContextRowView, SessionRequestRowView, SessionTraceRowView};
use super::tokens_note;
use crate::repositories::analytics::session_detail::{
    SessionContextRow, SessionRequestRow, SessionTraceRow,
};
use systemprompt_web_admin_core::ssr::entity_urls::{
    context_detail_url, request_detail_url, trace_detail_url,
};
use systemprompt_web_admin_core::ssr::format::{
    format_cost, format_duration_ms, local_time, short_id,
};

pub(super) fn context_view(c: &SessionContextRow) -> SessionContextRowView {
    SessionContextRowView {
        context_id: c.context_id.clone(),
        context_id_short: short_id(c.context_id.as_str()),
        context_url: context_detail_url(&c.context_id),
        name: c.name.clone().unwrap_or_else(|| "—".into()),
        request_count: c.request_count,
        last_request_at: c.last_request_at.map(|t| t.to_rfc3339()),
        last_request_at_local: c.last_request_at.map(local_time),
        model: c.model.clone(),
        total_tokens: c.total_input_tokens
            + c.total_cache_read_tokens
            + c.total_cache_creation_tokens
            + c.total_output_tokens,
        total_input_tokens: c.total_input_tokens,
        total_cache_read_tokens: c.total_cache_read_tokens,
        total_cache_creation_tokens: c.total_cache_creation_tokens,
        total_output_tokens: c.total_output_tokens,
        cost_microdollars: c.cost_microdollars,
        token_display: tokens_note(
            c.total_input_tokens + c.total_cache_read_tokens + c.total_cache_creation_tokens,
            c.total_cache_read_tokens,
            c.total_output_tokens,
        ),
        cost_display: format_cost(c.cost_microdollars),
        error_count: c.error_count,
    }
}

pub(super) fn trace_view(t: &SessionTraceRow) -> SessionTraceRowView {
    let duration_ms = match (t.started_at, t.ended_at) {
        (Some(s), Some(e)) => Some((e - s).num_milliseconds().max(0)),
        _ => None,
    };
    SessionTraceRowView {
        trace_id: t.trace_id.clone(),
        trace_id_short: short_id(t.trace_id.as_str()),
        trace_url: trace_detail_url(&t.trace_id),
        request_count: t.request_count,
        error_count: t.error_count,
        started_at_local: t.started_at.map(local_time),
        duration_display: duration_ms.map_or_else(|| "—".to_owned(), format_duration_ms),
    }
}

pub(super) fn request_view(r: &SessionRequestRow) -> SessionRequestRowView {
    SessionRequestRowView {
        id: r.id.clone(),
        id_short: short_id(r.id.as_str()),
        request_url: request_detail_url(&r.id),
        context_id: r.context_id.clone(),
        context_id_short: r.context_id.as_ref().map(|c| short_id(c.as_str())),
        context_url: r.context_id.as_ref().map(context_detail_url),
        trace_id: r.trace_id.clone(),
        trace_id_short: r.trace_id.as_ref().map(|t| short_id(t.as_str())),
        trace_url: r.trace_id.as_ref().map(trace_detail_url),
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_error: r.status == "failed",
        is_rejected: r.status == "rejected",
        latency_ms: r.latency_ms,
        input_tokens: r.input_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_creation_tokens: r.cache_creation_tokens,
        output_tokens: r.output_tokens,
        cost_microdollars: r.cost_microdollars,
        created_at: r.created_at.to_rfc3339(),
        latency_display: r
            .latency_ms
            .map_or_else(|| "—".to_owned(), |ms| format!("{ms}ms")),
        cost_display: format_cost(r.cost_microdollars),
        created_at_local: local_time(r.created_at),
    }
}

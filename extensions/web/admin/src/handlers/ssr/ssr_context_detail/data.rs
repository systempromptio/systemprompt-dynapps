//! Loads the conversation reader's rows: the request rollup, then the message
//! and tool-call bodies of the requests the transcript is built from, plus the
//! session's touched entities.
//!
//! Loading runs in two phases because message and tool-call bodies are fetched
//! by request id: the request rollup names the handful of requests the
//! transcript is built from. Tool calls are the one load that fails the page
//! instead of degrading to empty — no KPI counts them, so an empty list would
//! silently redact every tool invocation from the evidence.

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

use crate::handlers::ssr::transcript_view::transcript_request_ids;

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
    pub(super) kpis: ContextKpis,
    pub(super) requests: Vec<ContextRequestRow>,
    pub(super) messages: Vec<ContextMessageRow>,
    pub(super) tool_calls: Vec<ContextToolCallRow>,
    pub(super) entity_links: Vec<SessionEntityLink>,
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

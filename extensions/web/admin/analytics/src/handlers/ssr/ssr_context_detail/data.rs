//! Loads the conversation reader's rows: the request rollup, then the message
//! and tool-call bodies of the requests the transcript is built from, plus the
//! session's touched entities.
//!
//! Loading runs in two phases because message and tool-call bodies are fetched
//! by request id: the request rollup names the handful of requests the
//! transcript is built from. Tool calls are the one load that fails the page
//! instead of degrading to empty — no KPI counts them, so an empty list would
//! silently redact every tool invocation from the evidence. Every other read
//! that fails is recorded on the page's fault list.

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

use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::types::conversation_analytics::SessionEntityLink;

use crate::handlers::ssr::transcript_view::transcript_request_ids;

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
    faults: &mut Faults,
) -> Result<TranscriptRead, sqlx::Error> {
    let (kpis, requests) = load_request_rollup(pool, context_id, faults).await;
    let ids = transcript_request_ids(&requests);
    let (messages_res, tool_calls_res) = tokio::join!(
        list_messages_for_requests(pool, &ids.messages),
        list_tool_calls_for_requests(pool, &ids.tool_calls),
    );
    let messages = faults.take("messages", messages_res);
    let tool_calls = tool_calls_res?;
    let entity_links = load_session_entity_links(pool, header, faults).await;
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
    faults: &mut Faults,
) -> (ContextKpis, Vec<ContextRequestRow>) {
    let (kpis_res, requests_res) = tokio::join!(
        get_context_kpis(pool, context_id),
        list_context_requests(pool, context_id),
    );
    (
        faults.take("conversation totals", kpis_res),
        faults.take("request rollup", requests_res),
    )
}

async fn load_session_entity_links(
    pool: &PgPool,
    header: &ContextHeader,
    faults: &mut Faults,
) -> Vec<SessionEntityLink> {
    let Some(session_id) = header.session_id.as_ref() else {
        return Vec::new();
    };
    faults.take(
        "touched entities",
        list_session_entity_links(pool, session_id).await,
    )
}

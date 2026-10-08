//! `/admin/history/conversations/{context_id}` — one conversation, readable by
//! the person who had it.
//!
//! The admin reader at `/admin/contexts/{id}` renders the same transcript, but
//! it is admin-gated and redirects everyone else away, which is the second
//! reason a user with plenty of gateway traffic saw "no conversations". This
//! page answers the same question for the owner, with the gateway's transcript
//! framing stripped and every body run through the credential redactor.
//! Operational identifiers — trace, request, provider request, route match —
//! are deliberately absent; an admin follows the link to the full context view
//! for those.
//!
//! A conversation outside the viewer's scope answers 404 with the same message
//! as a missing id, so the URL is no oracle for other people's conversation
//! ids.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use crate::handlers::ssr::conversation_header::{
    ConversationStatsView, StatusBadgeView, resolve_title, stats_view, status_badge,
    timeline_display,
};
use crate::handlers::ssr::transcript_view::{
    ConversationView, TranscriptOptions, build_conversation, transcript_request_ids,
};
use crate::repositories::analytics::context_detail::{
    find_context_header, get_context_kpis, list_context_requests, list_messages_for_requests,
};
use crate::repositories::analytics::context_tool_calls::list_tool_calls_for_requests;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::conversations::get_history_scope;
use systemprompt_web_admin_core::ssr::entity_urls::context_detail_url;
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Serialize)]
struct ConversationPageContext {
    page: &'static str,
    title: String,
    stats: ConversationStatsView,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_badge: Option<StatusBadgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeline: Option<String>,
    conversation: ConversationView,
    admin_url: Option<String>,
    export: systemprompt_web_admin_core::export::view::ExportView,
}

fn not_found() -> AdminError {
    AdminError::NotFound("No conversation matches that id.".to_owned())
}

pub(crate) async fn history_conversation_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> AdminHtmlResult<Response> {
    let Ok(context_id) = ContextId::try_new(context_id.trim()) else {
        return Err(not_found().into());
    };

    let Some(header) = find_context_header(&pool, &context_id).await? else {
        return Err(not_found().into());
    };

    let scope = get_history_scope(&pool, &shell.user).await?;
    let Some(owner) = header.user_id.as_ref().filter(|o| scope.may_view(o)) else {
        return Err(not_found().into());
    };
    if owner != &shell.user.user_id {
        record_cross_user_read(&shell.user, owner, &context_id);
    }

    let (kpis_res, requests_res) = tokio::join!(
        get_context_kpis(&pool, &context_id),
        list_context_requests(&pool, &context_id),
    );
    let kpis = kpis_res?;
    let requests = requests_res?;

    let ids = transcript_request_ids(&requests);
    let (messages_res, tool_calls_res) = tokio::join!(
        list_messages_for_requests(&pool, &ids.messages),
        list_tool_calls_for_requests(&pool, &ids.tool_calls),
    );
    let messages = messages_res?;
    let tool_calls = tool_calls_res?;

    let conversation = build_conversation(
        &messages,
        &tool_calls,
        &requests,
        TranscriptOptions::owner_facing(),
    );

    let data = ConversationPageContext {
        page: "history",
        title: resolve_title(&header, &conversation),
        stats: stats_view(&kpis),
        status_badge: status_badge(header.hook_status.as_deref(), kpis.error_count),
        timeline: timeline_display(kpis.first_request_at, kpis.last_request_at),
        conversation,
        admin_url: shell.user.is_admin.then(|| context_detail_url(&context_id)),
        export: systemprompt_web_admin_core::export::view::ExportView::conversation(
            &context_id,
            shell.user.has_scoped_console(),
        ),
    };

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "history-conversation",
            &data,
            &shell.user,
            &shell.marketplace,
        ),
    )
}

fn record_cross_user_read(viewer: &UserContext, owner: &UserId, context_id: &ContextId) {
    tracing::info!(
        target: "history_access",
        viewer = %viewer.user_id,
        owner = %owner,
        context_id = %context_id,
        full_view = viewer.is_console,
        "conversation read by someone other than its owner"
    );
}

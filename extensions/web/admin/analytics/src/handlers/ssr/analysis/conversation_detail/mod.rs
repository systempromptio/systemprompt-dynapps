//! `/admin/analysis/conversations/{context_id}` — one conversation, every
//! plane: the record's header and KPIs, per-turn charts (tokens, cost,
//! latency), the turn ledger, every tool call from the ledger, every
//! governance decision, every safety finding, the skills invoked with the
//! marketplace version served at the time — and the judge's one label with
//! its rationale, plus the button that asks for it again.
//!
//! An owner outside the caller's view answers 404 like a missing id, so the
//! URL is no oracle for other people's ids.

mod context;
mod figures;
mod rows;
mod views;

use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::repositories::analysis::conversations::ConversationFactRow;
use crate::repositories::analysis::conversations::detail::{
    find_conversation_facts, insert_manual_judgement, refresh_conversation_facts,
};
use crate::repositories::analysis::conversations::planes::{
    list_conversation_decisions, list_conversation_safety_findings, list_conversation_skills,
    list_conversation_tool_calls, list_conversation_turns,
};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::scope::visibility::may_view;
use systemprompt_web_admin_core::ssr::analysis_urls::analysis_conversation_url;
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::types::UserContext;

use context::{Planes, build_context};

fn parse_context(raw: &str) -> Result<ContextId, AdminError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return Err(AdminError::BadRequest(
            "A conversation id is required.".to_owned(),
        ));
    }
    ContextId::try_new(trimmed)
        .map_err(|_invalid| AdminError::BadRequest("That is not a conversation id.".to_owned()))
}

async fn load_visible(
    pool: &PgPool,
    user: &UserContext,
    context_id: &ContextId,
) -> Result<ConversationFactRow, AdminError> {
    let not_found = || AdminError::NotFound("No conversation matches that id.".to_owned());
    if !refresh_conversation_facts(pool, context_id).await? {
        return Err(not_found());
    }
    let row = find_conversation_facts(pool, context_id)
        .await?
        .ok_or_else(not_found)?;
    if !may_view(pool, user, Some(&row.facts.user_id)).await? {
        return Err(not_found());
    }
    Ok(row)
}

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.has_scoped_console() {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let context_id = parse_context(&context_id)?;
    let row = load_visible(&pool, &shell.user, &context_id).await?;
    let session = row.facts.client_session_id.as_deref();
    let (turns, tools, decisions, skills, safety) = tokio::join!(
        list_conversation_turns(&pool, &context_id),
        list_conversation_tool_calls(&pool, &context_id, session),
        list_conversation_decisions(&pool, &context_id, session),
        list_conversation_skills(&pool, &context_id),
        list_conversation_safety_findings(&pool, &context_id),
    );
    let (turns, tools, decisions, skills, safety) = (turns?, tools?, decisions?, skills?, safety?);
    let context = build_context(
        &row,
        Planes {
            turns,
            tools,
            decisions,
            skills,
            safety,
        },
        shell.user.is_console,
    );
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "analysis-conversation",
            &context,
            &shell.user,
            &shell.marketplace,
        ),
    )
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct JudgeNowForm {
    pub back: Option<String>,
}

pub(crate) async fn judge_now(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
    headers: HeaderMap,
    Form(form): Form<JudgeNowForm>,
) -> AdminHtmlResult<Redirect> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    systemprompt_web_admin_core::request::require_write_origin(&headers)?;
    let context_id = parse_context(&context_id)?;
    load_visible(&pool, &shell.user, &context_id).await?;
    insert_manual_judgement(&pool, &context_id, &shell.user.user_id).await?;
    tracing::info!(
        context_id = %context_id,
        requested_by = %shell.user.user_id,
        "conversation queued for the judge from the console"
    );
    let back = form
        .back
        .filter(|b| b.starts_with("/admin/analysis/"))
        .unwrap_or_else(|| analysis_conversation_url(&context_id));
    Ok(Redirect::to(&back))
}

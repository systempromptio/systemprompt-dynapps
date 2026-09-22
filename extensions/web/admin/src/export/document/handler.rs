//! `GET /admin/export/conversation/{context_id}` — one conversation as JSON
//! or Markdown — and `GET /admin/export/conversations` — a selected or
//! filtered set as JSON Lines, one bundle per line.
//!
//! Visibility is the detail pages' rule: a context outside the caller's
//! scope answers 404, the same as a missing id, so the URL is no oracle for
//! other people's conversation ids. A viewer without a console seat gets the
//! owner-facing rendering: gateway framing stripped and credentials redacted,
//! exactly as `/admin/history` shows them.

use std::sync::Arc;

use axum::extract::{Extension, OriginalUri, Path, State};
use axum::http::header;
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use super::bundle::{ConversationBundle, load_conversation_bundle};
use super::markdown;
use super::selection::resolve_selection;
use crate::error::{AdminError, AdminResult};
use crate::export::model::ExportContext;
use crate::handlers::ssr::transcript_view::TranscriptOptions;
use crate::repositories::analytics::context_detail::find_context_header;
use crate::repositories::analytics::conversations::history_scope_for;
use crate::repositories::scope::visibility::may_view;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DocumentFormat {
    Json,
    Markdown,
}

#[derive(Debug, Default, Deserialize)]
struct DocumentQuery {
    format: Option<DocumentFormat>,
}

fn not_found() -> AdminError {
    AdminError::NotFound("No conversation matches that id.".to_owned())
}

// Why: console readers see the stored text as it is; everyone else is an
// owner reading their own conversation and gets the owner-facing treatment.
fn options_for(user: &UserContext) -> TranscriptOptions {
    if user.has_scoped_console() {
        TranscriptOptions::default()
    } else {
        TranscriptOptions::owner_facing()
    }
}

async fn visible(pool: &PgPool, user: &UserContext, context_id: &ContextId) -> AdminResult<bool> {
    let Some(header) = find_context_header(pool, context_id).await? else {
        return Ok(false);
    };
    let owner = header.user_id.as_ref();
    if user.has_scoped_console() {
        return Ok(may_view(pool, user, owner).await?);
    }
    Ok(owner.is_some_and(|o| history_scope_for(user).may_view(o)))
}

fn attachment(content_type: &str, filename: &str, body: String) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(body.into())
        .unwrap_or_default()
}

pub(crate) async fn export_conversation(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let Ok(context_id) = ContextId::try_new(context_id.trim()) else {
        return Err(not_found());
    };
    if !visible(&pool, &user, &context_id).await? {
        return Err(not_found());
    }
    let query = axum::extract::Query::<DocumentQuery>::try_from_uri(&uri)
        .map(|q| q.0)
        .map_err(|e| AdminError::BadRequest(e.body_text()))?;
    let bundle = load_conversation_bundle(&pool, &context_id, options_for(&user))
        .await?
        .ok_or_else(not_found)?;
    let short = context_id.as_str().get(..8).unwrap_or_default();
    Ok(match query.format.unwrap_or(DocumentFormat::Json) {
        DocumentFormat::Json => attachment(
            "application/json; charset=utf-8",
            &format!("conversation-{short}.json"),
            serde_json::to_string_pretty(&bundle).map_err(AdminError::internal)?,
        ),
        DocumentFormat::Markdown => attachment(
            "text/markdown; charset=utf-8",
            &format!("conversation-{short}.md"),
            markdown::render(&bundle),
        ),
    })
}

// Why: only the console exports sets — the selection resolvers read the
// list pages' scoped queries, and those pages are console pages.
pub(crate) async fn export_conversations(
    Extension(user): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    if !user.has_scoped_console() {
        return Err(AdminError::Forbidden("Console access required.".to_owned()));
    }
    let ctx = ExportContext::new(&pool, &user, &managed, uri);
    let selection = resolve_selection(&pool, &user, &ctx).await?;
    let opts = options_for(&user);
    let mut body = String::new();
    for context_id in &selection.context_ids {
        if !may_view(&pool, &user, owner_of(&pool, context_id).await?.as_ref()).await? {
            continue;
        }
        if let Some(bundle) = load_conversation_bundle(&pool, context_id, opts).await? {
            push_line(&mut body, &bundle)?;
        }
    }
    let filename = format!(
        "conversations-{}.jsonl",
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    let mut response = attachment("application/x-ndjson; charset=utf-8", &filename, body);
    if selection.capped() {
        response
            .headers_mut()
            .insert("x-export-capped", header::HeaderValue::from_static("true"));
    }
    Ok(response)
}

async fn owner_of(
    pool: &PgPool,
    context_id: &ContextId,
) -> AdminResult<Option<systemprompt::identifiers::UserId>> {
    Ok(find_context_header(pool, context_id)
        .await?
        .and_then(|h| h.user_id))
}

fn push_line(body: &mut String, bundle: &ConversationBundle) -> AdminResult<()> {
    body.push_str(&serde_json::to_string(bundle).map_err(AdminError::internal)?);
    body.push('\n');
    Ok(())
}

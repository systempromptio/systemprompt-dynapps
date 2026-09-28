//! `/admin/contexts/{context_id}` — the conversation reader.
//!
//! Renders the title bar, stat strip and one of three tabs (`?tab=`): the
//! turn-by-turn conversation, the request rollup with each request's kind,
//! and what the session touched.
//!
//! An owner outside the caller's view gets the same 404 as a missing id, so
//! the URL is no oracle for other people's context ids.

mod context;
mod data;

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::error::AdminHtmlResult;
use crate::repositories::analytics::context_detail::find_context_header;
use crate::repositories::scope::may_view;

use crate::handlers::ssr::page::Page;
use data::{build_detail_data, load_transcript, resolve_tab};

#[derive(Debug, Deserialize)]
pub(crate) struct ContextTabQuery {
    tab: Option<String>,
}

fn context_not_found() -> AdminError {
    AdminError::NotFound(
        "No conversation, AI request, or message rows match that context id.".to_owned(),
    )
}

// Why: core's `ContextId::new` panics on anything that is not a UUID, and
// this segment comes straight off the URL.
fn parse_context_id(raw: &str) -> Result<ContextId, AdminError> {
    ContextId::try_new(raw.trim()).map_err(|error| {
        tracing::debug!(%error, "context id segment is not a uuid");
        context_not_found()
    })
}

pub(crate) async fn context_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
    Query(query): Query<ContextTabQuery>,
) -> AdminHtmlResult<Response> {
    let context_id = parse_context_id(&context_id)?;

    let header = find_context_header(&pool, &context_id).await?;
    let visible = match &header {
        Some(h) => may_view(&pool, &shell.user, h.user_id.as_ref()).await?,
        None => false,
    };
    let Some(header) = header.filter(|_| visible) else {
        return Err(context_not_found().into());
    };

    let transcript = load_transcript(&pool, &header, &context_id).await?;
    let data = build_detail_data(&header, &transcript, resolve_tab(query.tab.as_deref()));

    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "context-detail",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

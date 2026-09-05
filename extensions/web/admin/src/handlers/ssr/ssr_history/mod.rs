//! `/admin/history` — a user's own conversation history, searchable.
//!
//! The one analytics surface a non-admin may reach: every viewer sees their
//! own conversations, and admin/auditor keep the unrestricted view. Both ways
//! a conversation gets recorded are listed — the Claude Code session
//! transcript written by the stop hook, and the gateway conversation grouped
//! out of `ai_requests` — because a user who only ever calls `/v1/messages`
//! has no transcript rows at all and used to be shown an empty page. The scope
//! is resolved server-side per request; asking for a `user_id` outside it is a
//! 403. Snippets pass through the transcript redactor before rendering, and
//! raw bodies stay on the existing admin/auditor-gated endpoint.

mod context;
mod conversation;

pub(crate) use conversation::history_conversation_page;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::transcript_view::short_id;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::conversations::{
    HistoryItem, HistoryScope, HistorySource, history_scope_for, list_history_items, redact_text,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use context::{HistoryPageContext, HistoryRowView};

const PAGE_SIZE: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HistoryQuery {
    q: Option<String>,
    user_id: Option<UserId>,
    page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct HistorySearchItem {
    source: &'static str,
    session_id: Option<SessionId>,
    context_id: Option<ContextId>,
    user_id: UserId,
    ai_title: Option<String>,
    preview: Option<String>,
    model: Option<String>,
    started_at: Option<String>,
    captured_at: String,
    entries_counted: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    cost_microdollars: i64,
    rank: Option<f32>,
    snippet: Option<String>,
    detail_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct HistorySearchEnvelope {
    items: Vec<HistorySearchItem>,
    total: i64,
    page: i64,
    page_size: i64,
}

struct HistorySlice {
    scope: HistoryScope,
    items: Vec<HistoryItem>,
    total: i64,
    page: i64,
}

async fn fetch_history_slice(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &HistoryQuery,
) -> Result<HistorySlice, AdminError> {
    let scope = history_scope_for(user_ctx);

    let target = query
        .user_id
        .as_ref()
        .filter(|u| !u.as_str().trim().is_empty());
    let scope_ids = match target {
        Some(target_id) => {
            if !scope.may_view(target_id) {
                return Err(AdminError::Forbidden(
                    "You may only view conversation history within your own scope.".to_owned(),
                ));
            }
            Some(vec![target_id.as_str().to_owned()])
        },
        None => scope.user_ids(),
    };

    let page = query.page.unwrap_or(0).max(0);
    let (items, total) = list_history_items(
        pool,
        scope_ids.as_deref(),
        query.q.as_deref(),
        PAGE_SIZE,
        page * PAGE_SIZE,
    )
    .await?;
    Ok(HistorySlice {
        scope,
        items,
        total,
        page,
    })
}

pub(crate) async fn history_search(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query).await?;
    let items = slice
        .items
        .into_iter()
        .map(|item| HistorySearchItem {
            source: item.source.label(),
            detail_url: detail_url(&item, &user_ctx),
            session_id: item.session_id.clone(),
            context_id: item.context_id.clone(),
            user_id: item.user_id,
            ai_title: item.title,
            preview: item.preview.map(|s| redact_text(&s).0),
            model: item.model,
            started_at: item.started_at.map(|t| t.to_rfc3339()),
            captured_at: item.last_at.to_rfc3339(),
            entries_counted: item.turns,
            total_input_tokens: item.total_input_tokens,
            total_output_tokens: item.total_output_tokens,
            cost_microdollars: item.cost_microdollars,
            rank: item.rank,
            snippet: item.snippet.map(|s| redact_text(&s).0),
        })
        .collect();
    Ok(Json(HistorySearchEnvelope {
        items,
        total: slice.total,
        page: slice.page,
        page_size: PAGE_SIZE,
    })
    .into_response())
}

pub(crate) async fn history_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminHtmlResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query).await?;

    let rows: Vec<HistoryRowView> = slice
        .items
        .iter()
        .map(|item| row_view(item, &user_ctx))
        .collect();

    let window = PageWindow::new(
        slice.page,
        PAGE_SIZE,
        slice.total,
        i64::try_from(rows.len()).unwrap_or(PAGE_SIZE),
        "conversations",
    );
    let data = HistoryPageContext {
        page: "history",
        title: "My Conversations",
        search_query: query.q.clone().unwrap_or_default(),
        filter_user_id: query.user_id.as_ref().map(|u| u.as_str().to_owned()),
        viewer_is_admin: user_ctx.is_admin,
        scope_label: scope_label(&slice.scope),
        has_rows: !rows.is_empty(),
        rows,
        pagination: build_pagination(&query, window),
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("My conversations"),
        ],
    };
    Ok(super::render_typed_page(
        &engine, "history", &data, &user_ctx, &mkt_ctx,
    ))
}

fn scope_label(scope: &HistoryScope) -> String {
    match scope {
        HistoryScope::All => "all users".to_owned(),
        HistoryScope::Users(_) => "your own conversations".to_owned(),
    }
}

// Why: a gateway conversation has an owner-facing detail page, so every viewer
// gets the link. A transcript's only detail page is the admin session view.
fn detail_url(item: &HistoryItem, viewer: &UserContext) -> Option<String> {
    match item.source {
        HistorySource::Gateway => item.context_id.as_ref().map(|c| {
            format!(
                "/admin/history/conversations/{}",
                urlencoding::encode(c.as_str())
            )
        }),
        HistorySource::Transcript => viewer
            .is_admin
            .then(|| item.session_id.as_ref().map(session_detail_url))
            .flatten(),
    }
}

fn row_identity(item: &HistoryItem) -> String {
    item.session_id
        .as_ref()
        .map(|s| s.as_str().to_owned())
        .or_else(|| item.context_id.as_ref().map(|c| c.as_str().to_owned()))
        .unwrap_or_default()
}

fn row_view(item: &HistoryItem, viewer: &UserContext) -> HistoryRowView {
    let when = item.last_at;
    let identity = row_identity(item);
    let short = short_id(&identity);
    let is_gateway = item.source == HistorySource::Gateway;
    HistoryRowView {
        source: item.source.label(),
        is_gateway,
        session_id: item.session_id.clone(),
        title: item
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| {
                if is_gateway {
                    format!("Conversation {short}")
                } else {
                    format!("Session {short}")
                }
            }),
        short_id: short,
        user_id: item.user_id.clone(),
        is_own: item.user_id == viewer.user_id,
        model: item.model.clone(),
        when_local: when
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        entries_counted: item.turns,
        tokens_display: format!(
            "{} in / {} out",
            item.total_input_tokens, item.total_output_tokens
        ),
        cost_display: is_gateway.then(|| format_cost(item.cost_microdollars)),
        preview: item.preview.as_deref().map(|s| redact_text(s).0),
        snippet: item.snippet.as_deref().map(|s| redact_text(s).0),
        detail_url: detail_url(item, viewer),
    }
}

fn build_pagination(query: &HistoryQuery, window: PageWindow) -> Pagination {
    let mut parts: Vec<String> = Vec::new();
    if let Some(q) = query.q.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("q={}", urlencoding::encode(q)));
    }
    if let Some(u) = query
        .user_id
        .as_ref()
        .map(UserId::as_str)
        .filter(|s| !s.is_empty())
    {
        parts.push(format!("user_id={}", urlencoding::encode(u)));
    }
    let prefix = if parts.is_empty() {
        "/admin/history?".to_owned()
    } else {
        format!("/admin/history?{}&", parts.join("&"))
    };
    let page = window.index;
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

//! `/admin/history` and `/admin/conversations` — the same searchable listing
//! at two scopes.
//!
//! `/admin/history` is "My conversations" and shows the viewer's own, whoever
//! is looking. `/admin/conversations` is the admin-gated org-wide listing, and
//! carries the User column and the per-user filter. One set of rows, one
//! search, one pagination; [`HistoryView`] is the whole of the difference.
//!
//! The one analytics surface a non-admin may reach: every viewer sees their
//! own conversations, and admin/auditor keep the unrestricted view. Both ways
//! a conversation gets recorded are listed — the Claude Code session
//! transcript written by the stop hook, and the gateway conversation grouped
//! out of `ai_requests` — because a user who only ever calls `/v1/messages`
//! has no transcript rows at all and used to be shown an empty page. The scope
//! is resolved server-side per request; asking for a `user_id` outside it is a
//! 403. Snippets pass through the transcript redactor before rendering, and
//! raw bodies stay on the existing admin/auditor-gated endpoint. Side calls
//! — cache probes and utility calls — are hidden unless `?side=1`.

mod context;
mod conversation;
mod kind;
mod view;

pub(crate) use context::HistoryRowView;
pub(crate) use conversation::history_conversation_page;
pub(crate) use kind::HistoryView;
pub use view::command_name;
pub(crate) use view::row_view;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::conversations::{
    HistoryFilter, HistoryItem, HistoryScope, list_history_items, redact_text,
};
use crate::types::UserContext;

use crate::handlers::ssr::page::Page;
use context::HistoryPageContext;
use view::{build_pagination, detail_url, scope_label, side_toggle_url};

const PAGE_SIZE: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HistoryQuery {
    q: Option<String>,
    user_id: Option<UserId>,
    page: Option<i64>,
    side: Option<String>,
}

impl HistoryQuery {
    fn show_side(&self) -> bool {
        self.side.as_deref() == Some("1")
    }
}

#[derive(Debug, Serialize)]
struct HistorySearchItem {
    session_id: Option<SessionId>,
    context_id: ContextId,
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
    side_call_count: i64,
    detail_url: String,
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
    view: HistoryView,
) -> Result<HistorySlice, AdminError> {
    let scope = view.scope(user_ctx);

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
        HistoryFilter {
            scope_user_ids: scope_ids.as_deref(),
            search: query.q.as_deref(),
            include_side_calls: query.show_side(),
        },
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

// Why: the export is the listing the reader sees — their own scope, the same
// search — through the page's own query type.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: HistoryQuery,
    view: HistoryView,
    limit: i64,
) -> Result<(Vec<HistoryItem>, i64), AdminError> {
    let query = HistoryQuery {
        page: None,
        ..query
    };
    let scope = view.scope(user_ctx);
    let target = query
        .user_id
        .as_ref()
        .filter(|u| !u.as_str().trim().is_empty());
    let scope_ids = match target {
        Some(target_id) if scope.may_view(target_id) => Some(vec![target_id.as_str().to_owned()]),
        Some(_) => {
            return Err(AdminError::Forbidden(
                "You may only export conversation history within your own scope.".to_owned(),
            ));
        },
        None => scope.user_ids(),
    };
    Ok(list_history_items(
        pool,
        HistoryFilter {
            scope_user_ids: scope_ids.as_deref(),
            search: query.q.as_deref(),
            include_side_calls: query.show_side(),
        },
        limit,
        0,
    )
    .await?)
}

pub(crate) async fn history_search(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminResult<Response> {
    let slice = fetch_history_slice(&pool, &user_ctx, &query, HistoryView::Own).await?;
    let items = slice
        .items
        .into_iter()
        .map(|item| HistorySearchItem {
            detail_url: detail_url(&item, HistoryView::Own),
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
            side_call_count: item.side_call_count,
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
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminHtmlResult<Response> {
    render_listing(&shell, &pool, &query, HistoryView::Own).await
}

pub(crate) async fn conversations_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HistoryQuery>,
) -> AdminHtmlResult<Response> {
    render_listing(&shell, &pool, &query, HistoryView::Org).await
}

async fn render_listing(
    shell: &Page,
    pool: &PgPool,
    query: &HistoryQuery,
    view: HistoryView,
) -> AdminHtmlResult<Response> {
    let user_ctx = &shell.user;
    let slice = fetch_history_slice(pool, user_ctx, query, view).await?;

    let rows: Vec<HistoryRowView> = slice
        .items
        .iter()
        .map(|item| row_view(item, user_ctx, view))
        .collect();

    let window = PageWindow::new(
        slice.page,
        PAGE_SIZE,
        slice.total,
        i64::try_from(rows.len()).unwrap_or(PAGE_SIZE),
        "conversations",
    );
    let base = view.base_url();
    let data = HistoryPageContext {
        page: view.page_id(),
        title: view.title(),
        search_query: query.q.clone().unwrap_or_default(),
        filter_user_id: query.user_id.as_ref().map(|u| u.as_str().to_owned()),
        scope_label: scope_label(&slice.scope),
        has_rows: !rows.is_empty(),
        rows,
        show_side: query.show_side(),
        side_toggle_url: side_toggle_url(query, base),
        base_url: base,
        pagination: build_pagination(query, window, base),
        breadcrumbs: view.breadcrumbs(),
        export: crate::export::ExportView::single(
            view.dataset(),
            &crate::export::view::query_string(&[
                ("q", query.q.as_deref()),
                ("user_id", query.user_id.as_ref().map(UserId::as_str)),
                ("side", query.side.as_deref()),
            ]),
        ),
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        view.template(),
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

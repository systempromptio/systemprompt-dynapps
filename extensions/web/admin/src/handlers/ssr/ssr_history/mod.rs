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
mod search;
mod view;
mod window;

pub(crate) use context::HistoryRowView;
pub(crate) use conversation::history_conversation_page;
pub(crate) use kind::HistoryView;
pub(crate) use search::history_search;
pub use view::command_name;
pub(crate) use view::row_view;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::export::model::ExportWindow;
use crate::export::view::TranscriptSource;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::conversations::{
    HistoryFilter, HistoryItem, HistoryScope, list_history_items,
};
use crate::types::UserContext;

use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;
use crate::handlers::ssr::page::Page;
use context::{HiddenInputView, HistoryPageContext};
use view::{build_pagination, scope_label, side_toggle_url, window_links};
use window::HistoryWindow;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct HistoryQuery {
    q: Option<String>,
    user_id: Option<UserId>,
    page: Option<i64>,
    side: Option<String>,
    // Why: the window — a preset `days`, or a custom `start`/`end` — bounds a
    // conversation's last activity; none of them means all time.
    days: Option<u32>,
    start: Option<String>,
    end: Option<String>,
}

impl HistoryQuery {
    fn show_side(&self) -> bool {
        self.side.as_deref() == Some("1")
    }
}

struct HistorySlice {
    scope: HistoryScope,
    items: Vec<HistoryItem>,
    total: i64,
    page: i64,
}

// Why: the viewer's scope, narrowed to one user when the query names one they
// may see; naming one outside it is a 403 on the page and the export alike.
fn scope_user_ids(
    scope: &HistoryScope,
    query: &HistoryQuery,
    verb: &str,
) -> Result<Option<Vec<String>>, AdminError> {
    let target = query
        .user_id
        .as_ref()
        .filter(|u| !u.as_str().trim().is_empty());
    match target {
        Some(target_id) if scope.may_view(target_id) => {
            Ok(Some(vec![target_id.as_str().to_owned()]))
        },
        Some(_) => Err(AdminError::Forbidden(format!(
            "You may only {verb} conversation history within your own scope."
        ))),
        None => Ok(scope.user_ids()),
    }
}

async fn fetch_history_slice(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &HistoryQuery,
    view: HistoryView,
) -> Result<HistorySlice, AdminError> {
    let scope = view.scope(user_ctx);
    let scope_ids = scope_user_ids(&scope, query, "view")?;
    let (since, until) = HistoryWindow::of(query).bounds();
    let page = query.page.unwrap_or(0).max(0);
    let (items, total) = list_history_items(
        pool,
        HistoryFilter {
            scope_user_ids: scope_ids.as_deref(),
            search: query.q.as_deref(),
            include_side_calls: query.show_side(),
            since,
            until,
        },
        DEFAULT_PAGE_SIZE,
        page * DEFAULT_PAGE_SIZE,
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
// search — through the page's own query type. The dialog's resolved window
// wins; without one the rows follow the page's own window.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: HistoryQuery,
    view: HistoryView,
    window: Option<ExportWindow>,
    limit: i64,
) -> Result<(Vec<HistoryItem>, i64), AdminError> {
    let scope = view.scope(user_ctx);
    let scope_ids = scope_user_ids(&scope, &query, "export")?;
    let (since, until) = window.map_or_else(
        || HistoryWindow::of(&query).bounds(),
        |w| (Some(w.from), Some(w.to)),
    );
    Ok(list_history_items(
        pool,
        HistoryFilter {
            scope_user_ids: scope_ids.as_deref(),
            search: query.q.as_deref(),
            include_side_calls: query.show_side(),
            since,
            until,
        },
        limit,
        0,
    )
    .await?)
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
        DEFAULT_PAGE_SIZE,
        slice.total,
        i64::try_from(rows.len()).unwrap_or(DEFAULT_PAGE_SIZE),
        "conversations",
    );
    let base = view.base_url();
    let history_window = HistoryWindow::of(query);
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
        window_links: window_links(query, base),
        window_label: history_window.label(),
        window_inputs: history_window
            .pairs()
            .into_iter()
            .map(|(name, value)| HiddenInputView { name, value })
            .collect(),
        base_url: base,
        pagination: build_pagination(query, window, base),
        breadcrumbs: view.breadcrumbs(),
        export: export_view(query, view, history_window),
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        view.template(),
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

// Why: the export opens on the listing the reader is looking at — the same
// search, user, side calls and window — and offers the full record of every
// conversation it selects.
fn export_view(
    query: &HistoryQuery,
    view: HistoryView,
    window: HistoryWindow,
) -> crate::export::ExportView {
    let window_pairs = window.export_pairs();
    let mut pairs = vec![
        ("q", query.q.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("side", query.side.as_deref()),
    ];
    pairs.extend(window_pairs.iter().map(|(k, v)| (*k, Some(v.as_str()))));
    let source = match view {
        HistoryView::Org => TranscriptSource::Conversations,
        HistoryView::Own => TranscriptSource::History,
    };
    crate::export::ExportView::single(view.dataset(), &crate::export::view::query_string(&pairs))
        .with_transcripts(source)
}

//! `/admin/sessions` — the sessions list.
//!
//! A session is a conversation: one context, from whichever client recorded
//! it, holding every turn, tool call and side call it produced. This page is
//! the ledger — one row per conversation in the window with turns, tool
//! calls, cost and errors, for finding the expensive, failing or busy ones —
//! beside `/admin/conversations` (reading, transcript search) and
//! `/admin/contexts` (activity by person). It lists them under the caller's
//! scope and hands each one to the conversation reader at
//! `/admin/contexts/{id}`.
//!
//! Bound to the same `?preset=&from=&to=&user_id=&error_only=&sort=&dir=&page=`
//! contract as the other AI-activity list pages, plus the `?group=&project=`
//! scope every one of them shares and `?side=1` to include conversations made
//! of side calls alone.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::conversations::rows::{
    ConversationFilter, ConversationPage, ConversationPageMode, ConversationPageResult,
    ConversationSort, ConversationTotals, get_conversation_totals, load_conversation_page,
};
use systemprompt_web_admin_core::repositories::filter_options::{
    FilterOptions, get_filter_options,
};
use systemprompt_web_admin_core::repositories::scope::{ScopeRequest, SubjectScope};
use systemprompt_web_admin_core::ssr::entity_urls::session_detail_url;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::format::short_id;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::list_view::PageWindow;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod context;
pub(crate) mod export;
mod rows;
mod summary;
mod view;

use context::{CurrentSessionView, FilterRibbon, SessionsListPageContext};
use systemprompt_web_admin_core::ssr::list_view::DEFAULT_PAGE_SIZE;
use systemprompt_web_admin_core::ssr::page::Page;

const BASE_URL: &str = "/admin/sessions";
#[derive(Debug, Deserialize)]
pub(crate) struct SessionListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<UserId>,
    pub error_only: Option<String>,
    pub side: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
}

pub(crate) async fn sessions_list_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SessionListQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let page = query.page.unwrap_or(0).max(0);

    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let subjects = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        &pool, &request,
    )
    .await?;

    let mut faults = Faults::new();
    let ctx = load_sessions_data(
        &pool,
        SessionScope {
            user: &shell.user,
            request: &request,
            subjects,
        },
        &query,
        SessionWindow { range, page },
        &mut faults,
    )
    .await;
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &shell.engine,
            "sessions",
            &ctx,
            Viewer::new(&shell.user, &shell.marketplace),
            &faults,
        ),
    )
}

#[derive(Clone, Copy)]
struct SessionWindow {
    range: TimeRange,
    page: i64,
}

struct SessionScope<'a> {
    user: &'a UserContext,
    request: &'a ScopeRequest,
    subjects: SubjectScope,
}

struct LoadedSessions {
    result: ConversationPageResult,
    previous: ConversationTotals,
    options: FilterOptions,
    filter: ConversationFilter,
    sort: ConversationSort,
    descending: bool,
}

async fn load_sessions(
    pool: &PgPool,
    scope: &SessionScope<'_>,
    query: &SessionListQuery,
    window: SessionWindow,
    faults: &mut Faults,
) -> LoadedSessions {
    let SessionWindow { range, page } = window;
    let filter = ConversationFilter {
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        subject_ids: scope.subjects.as_sql().map(<[String]>::to_vec),
        model: None,
        free_text: None,
        since: Some(range.from),
        until: Some(range.to),
        include_side_calls: query.side.as_deref() == Some("1"),
        error_only: query.error_only.as_deref() == Some("true"),
    };
    let totals_filter = ConversationFilter {
        since: Some(range.from - (range.to - range.from)),
        until: Some(range.from),
        ..filter.clone()
    };
    let sort = ConversationSort::parse_conversation_sort(query.sort.as_deref());
    let descending = query.dir.as_deref() != Some("asc");
    let conversation_page = ConversationPage {
        sort,
        descending,
        limit: DEFAULT_PAGE_SIZE,
        offset: page * DEFAULT_PAGE_SIZE,
    };

    let (list_res, totals_res, options_res) = tokio::join!(
        load_conversation_page(pool, &filter, conversation_page, ConversationPageMode::All),
        get_conversation_totals(pool, &totals_filter),
        get_filter_options(pool, range),
    );
    LoadedSessions {
        result: faults.take("conversation page", list_res),
        previous: faults.take("previous-window totals", totals_res),
        options: faults.take("filter options", options_res),
        filter,
        sort,
        descending,
    }
}

async fn load_sessions_data(
    pool: &PgPool,
    scope: SessionScope<'_>,
    query: &SessionListQuery,
    window: SessionWindow,
    faults: &mut Faults,
) -> SessionsListPageContext {
    let SessionWindow { range, page } = window;
    let loaded = load_sessions(pool, &scope, query, window, faults).await;
    let items = loaded.result.conversations;
    let current = loaded.result.totals;
    let total = current.conversations;
    let error_only = loaded.filter.error_only;
    let show_side = loaded.filter.include_side_calls;

    let preset = preset_str(query, range);
    let export_query = systemprompt_web_admin_core::export::view::query_string(&[
        ("preset", Some(preset.as_str())),
        ("from", Some(&range.from.to_rfc3339())),
        ("to", Some(&range.to.to_rfc3339())),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("error_only", query.error_only.as_deref()),
        ("side", query.side.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
    ]);
    let session_rows: Vec<_> = items.iter().map(rows::session_row).collect();
    let has_sessions = !session_rows.is_empty();
    let shown_rows = i64::try_from(session_rows.len()).unwrap_or(DEFAULT_PAGE_SIZE);
    let page_window = PageWindow::new(page, DEFAULT_PAGE_SIZE, total, shown_rows, "conversations");

    SessionsListPageContext {
        page: "sessions",
        title: "Sessions",
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::current("Sessions"),
        ],
        current: current_session_view(scope.user),
        time_range: view::time_range_context(range, &preset),
        filter_ribbon: FilterRibbon {
            base_url: BASE_URL,
            preserved: view::build_preserved(query, range, &preset),
            options: view::annotate_options(
                &loaded.options.users,
                loaded.filter.user_id.as_ref().map(UserId::as_str),
            ),
            chips: view::build_chips(query),
        },
        scope_filter: view::scope_filter(
            pool,
            scope.user,
            &view::SessionScopeFilterArgs {
                request: scope.request,
                query,
                range,
                preset: &preset,
            },
        )
        .await,
        stats: summary::stats_view(&current, &loaded.previous),
        sessions: session_rows,
        has_sessions,
        total_count: total,
        count_label: format!("{total} conversations"),
        pagination: view::build_pagination(query, page_window),
        sort_headers: summary::build_sort_headers(query, loaded.sort, loaded.descending),
        error_only,
        error_toggle_url: view::error_toggle_url(query, error_only),
        show_side,
        side_toggle_url: view::side_toggle_url(query, show_side),
        export: systemprompt_web_admin_core::export::view::ExportView::single(
            "sessions",
            &export_query,
        )
        .with_transcripts(
            systemprompt_web_admin_core::export::transcripts::TranscriptSource::Sessions,
        ),
    }
}

fn preset_str(query: &SessionListQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref()
        && !p.is_empty()
    {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    range.preset.as_str().to_owned()
}

fn current_session_view(user_ctx: &UserContext) -> CurrentSessionView {
    let session_url = user_ctx.session_id.as_ref().map(session_detail_url);
    let session_id = user_ctx.session_id.clone();
    CurrentSessionView {
        username: user_ctx.username.clone(),
        session_id_short: session_id.as_ref().map(|s| short_id(s.as_str())),
        session_url,
        session_id,
    }
}

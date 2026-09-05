//! `/admin/sessions` — the sessions list.
//!
//! A session is the unit an operator actually recognises: one interactive run,
//! holding every context, trace and AI request it produced. This page lists
//! them under the caller's scope and hands each one to `ssr_session_detail`.
//!
//! Bound to the same `?preset=&from=&to=&user_id=&error_only=&sort=&dir=&page=`
//! contract as the other AI-activity list pages, plus the `?group=&project=`
//! scope every one of them shares.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::short_id;
use crate::handlers::ssr::list_view::PageWindow;
use crate::repositories::analytics::sessions_list::{
    SessionListFilter, SessionListKpis, SessionPage, SessionSort, SessionSortColumn,
    SessionSortDir, get_session_list_kpis, list_sessions_paged,
};
use crate::repositories::governance::filter_options::get_filter_options;
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range};

mod context;
mod rows;
mod view;

use context::{CurrentSessionView, FilterRibbon, SessionsListPageContext};

const BASE_URL: &str = "/admin/sessions";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct SessionListQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub user_id: Option<UserId>,
    pub error_only: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
    pub group: Option<String>,
    pub project: Option<String>,
}

pub(crate) async fn sessions_list_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SessionListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let page = query.page.unwrap_or(0).max(0);

    let request =
        ScopeRequest::from_query(&user_ctx, query.group.as_deref(), query.project.as_deref());
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(&pool, &request).await?;

    let ctx = load_sessions_data(
        &pool,
        &user_ctx,
        SessionScope {
            request: &request,
            subjects,
        },
        &query,
        SessionWindow { range, page },
    )
    .await;
    Ok(super::render_typed_page(
        &engine, "sessions", &ctx, &user_ctx, &mkt_ctx,
    ))
}

// Why: the resolved window and page travel together — clippy's argument cap is
// the only reason they are not two parameters.
struct SessionWindow {
    range: TimeRange,
    page: i64,
}

// Why: the resolved user set and the request that produced it travel together —
// the queries bind the first, the scope form re-renders the second.
struct SessionScope<'a> {
    request: &'a ScopeRequest,
    subjects: SubjectScope,
}

async fn load_sessions_data(
    pool: &PgPool,
    user_ctx: &UserContext,
    scope: SessionScope<'_>,
    query: &SessionListQuery,
    window: SessionWindow,
) -> SessionsListPageContext {
    let SessionWindow { range, page } = window;
    let error_only = query.error_only.as_deref() == Some("true");
    let filter = SessionListFilter {
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        subject_ids: scope.subjects.as_sql().map(<[String]>::to_vec),
        error_only,
    };
    let sort = SessionSort {
        column: SessionSortColumn::parse(query.sort.as_deref()),
        dir: SessionSortDir::parse(query.dir.as_deref()),
    };
    let session_page = SessionPage {
        sort,
        limit: PAGE_SIZE,
        offset: page * PAGE_SIZE,
    };

    let (list_res, kpis_res, options_res) = tokio::join!(
        list_sessions_paged(pool, &filter, range, session_page),
        get_session_list_kpis(pool, &filter, range),
        get_filter_options(pool, range),
    );

    let (items, total) = list_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_sessions_paged failed");
        (Vec::new(), 0)
    });
    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "get_session_list_kpis failed");
        SessionListKpis::default()
    });
    let options = options_res.unwrap_or_default();

    let preset = preset_str(query, range);
    let session_rows: Vec<_> = items.iter().map(rows::session_row).collect();
    let has_sessions = !session_rows.is_empty();
    let shown_rows = i64::try_from(session_rows.len()).unwrap_or(PAGE_SIZE);
    let page_window = PageWindow::new(page, PAGE_SIZE, total, shown_rows, "sessions");

    SessionsListPageContext {
        page: "sessions",
        title: "Sessions",
        current: current_session_view(user_ctx),
        time_range: view::time_range_context(range, &preset),
        filter_ribbon: FilterRibbon {
            base_url: BASE_URL,
            preserved: view::build_preserved(query, range, &preset),
            options: view::annotate_options(
                &options.users,
                filter.user_id.as_ref().map(UserId::as_str),
            ),
            chips: view::build_chips(query),
        },
        scope_filter: view::scope_filter(
            pool,
            user_ctx,
            &view::ScopeFilterArgs {
                request: scope.request,
                query,
                range,
                preset: &preset,
            },
        )
        .await,
        stats: view::stats_view(&kpis),
        sessions: session_rows,
        has_sessions,
        total_count: total,
        count_label: format!("{total} sessions"),
        pagination: view::build_pagination(query, page_window),
        sort_headers: view::build_sort_headers(query, sort.column, sort.dir),
        error_only,
        error_toggle_url: view::error_toggle_url(query, error_only),
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
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_owned()
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

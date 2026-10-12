//! `/admin/farm/runs` — the runs explorer: every headless run across all
//! farms, filtered by farm, agent, user, status and trigger over a time range,
//! sorted and paged like the Trace Explorer. While any row on the page is
//! live the page polls `/admin/api/farm/runs` for the same KPI strip and
//! table, rendered from the same view model.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::list_view::{DEFAULT_PAGE_SIZE, TimeRangeContext};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

pub(crate) mod context;
mod options;
pub(crate) mod query;
pub(crate) mod view;

use crate::repositories::filters::PageWindow;
use crate::repositories::rows::RunRow;
use crate::repositories::summaries::RunStats;
use crate::repositories::{agents, farms, people, runs};
use context::{RunsFilterView, RunsPageContext};
use query::{BASE_URL, FRAGMENT_URL, RunsQuery, run_filter, run_sort};

pub(crate) fn time_range(query: &RunsQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}

fn preset_str(query: &RunsQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref().filter(|p| !p.is_empty()) {
        return p.to_owned();
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_owned();
    }
    range.preset.as_str().to_owned()
}

pub(crate) struct RunsRead {
    pub(crate) rows: Vec<RunRow>,
    pub(crate) total: i64,
    pub(crate) stats: RunStats,
}

pub(crate) async fn read_runs(
    pool: &PgPool,
    query: &RunsQuery,
    range: TimeRange,
) -> Result<RunsRead, sqlx::Error> {
    let filter = run_filter(query, range.from, range.to);
    let (_, sort) = run_sort(query);
    let window = PageWindow {
        limit: DEFAULT_PAGE_SIZE,
        offset: query.page_index() * DEFAULT_PAGE_SIZE,
    };
    let (list, stats) = tokio::join!(
        runs::list_runs_paged(pool, &filter, sort, window),
        runs::get_run_stats(pool, &filter),
    );
    let (rows, total) = list?;
    Ok(RunsRead {
        rows,
        total,
        stats: stats?,
    })
}

pub(crate) fn live_view(query: &RunsQuery, read: &RunsRead) -> context::RunsLiveView {
    view::live_view(
        query,
        &view::RunsPage {
            rows: &read.rows,
            total: read.total,
            stats: &read.stats,
            page: query.page_index(),
            column: run_sort(query).0,
        },
    )
}

pub(crate) async fn runs_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RunsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let range = time_range(&query);
    let preset = preset_str(&query, range);
    let mut faults = Faults::new();
    let (read, farm_rows, agent_rows, user_rows) = tokio::join!(
        read_runs(&pool, &query, range),
        farms::list_farms(&pool),
        agents::list_all_agents(&pool),
        people::list_runs_by_user(&pool, range.from, None),
    );
    let read = faults.take_with("runs", read, || RunsRead {
        rows: Vec::new(),
        total: 0,
        stats: RunStats::default(),
    });
    let farm_rows = faults.take("farms", farm_rows);
    let agent_rows = faults.take("agents", agent_rows);
    let user_rows = faults.take("people", user_rows);

    let ctx = RunsPageContext {
        page: "farm-runs",
        title: "Runs",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current("Runs"),
        ],
        time_range: TimeRangeContext {
            preset: preset.clone(),
            from: range.from.to_rfc3339(),
            to: range.to.to_rfc3339(),
            base_url: BASE_URL,
            query: "",
            rejected: range.rejected_bounds,
        },
        filters: RunsFilterView {
            base_url: BASE_URL,
            preserved: options::preserved(&query, range, &preset),
            farms: options::farm_options(&farm_rows, query.farm()),
            agents: options::agent_options(&agent_rows, query.agent()),
            users: options::user_options(&user_rows, query.user()),
            statuses: options::status_options(query.status_value()),
            triggers: options::trigger_options(query.trigger_value()),
            chips: options::chips(&query),
            clear_url: options::clear_url(&query),
        },
        total_count: read.total,
        fragment_url: FRAGMENT_URL,
        live: live_view(&query, &read),
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-runs",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

//! `/admin/analytics` — the site analytics dashboard.
//!
//! Four URL-driven tabs over one window: Overview (KPIs, request/cost trends,
//! model pie), Usage (top-users leaderboard, permission grant rate), Spend
//! (latency SLO split, usage anomalies), and Code (commit activity and AI
//! line deltas). Drill-down state — project, user, window, bucket — all lives
//! in the query string.
//!
//! Scoping: admin-only, so `?project=` is always honoured; the same
//! [`ProjectScope`] rule pins a non-admin to their own project everywhere.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analytics::site::SiteScope;
use crate::repositories::analytics::site::leaderboards::LeaderboardSort;
use crate::repositories::analytics::site::series::SeriesBucket;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::project_scope::ProjectScope;
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

mod context;
mod data;
mod filters;
mod urls;
mod view;
mod view_code;
mod view_models;
mod view_spend;
mod view_tables;

use context::{AnalyticsDashboardContext, DashboardTab, FiltersView};

const BASE_URL: &str = "/admin/analytics";
const PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct AnalyticsDashboardQuery {
    pub tab: Option<String>,
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub bucket: Option<String>,
    pub project: Option<String>,
    pub user_id: Option<systemprompt::identifiers::UserId>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub slo_ms: Option<i32>,
}

pub(crate) async fn analytics_dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = DashboardTab::from_query(query.tab.as_deref());
    let bucket = SeriesBucket::from_bucket_param(query.bucket.as_deref());
    let sort = LeaderboardSort::from_sort_param(query.sort.as_deref());
    let range = resolve_range(&query);
    let page = query.page.unwrap_or(0).max(0);

    let scope = resolve_scope(&user_ctx, &query);
    let slo_ms = crate::repositories::analytics::site::latency::resolve_slo_ms(query.slo_ms);

    let fetched = data::load_dashboard_data(
        &pool,
        data::DashboardQueryPlan {
            tab,
            scope: &scope,
            range,
            bucket,
            sort,
            page_size: PAGE_SIZE,
            offset: page * PAGE_SIZE,
            slo_ms,
        },
    )
    .await;

    let filters = filters::build_filters(&user_ctx, &query, &scope, bucket);
    let ctx = page_context(PageInput {
        query: &query,
        tab,
        range,
        bucket,
        page,
        filters,
        fetched: &fetched,
        slo_ms,
    });

    Ok(super::render_typed_page(
        &engine,
        "analytics-dashboard",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

// Why: a trends page defaults to a week, not the request log's 24h — daily
// buckets need more than one of them to read as a trend.
fn resolve_range(query: &AnalyticsDashboardQuery) -> TimeRange {
    let user_picked = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    if user_picked {
        parse_time_range(&TimeRangeQuery {
            from: query.from.clone(),
            to: query.to.clone(),
            preset: query.preset.clone(),
        })
    } else {
        preset_to_range(TimeRangePreset::Days7)
    }
}

fn resolve_scope(user_ctx: &UserContext, query: &AnalyticsDashboardQuery) -> SiteScope {
    SiteScope {
        project: ProjectScope::resolve(user_ctx, query.project.as_deref()),
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
    }
}

fn sort_links(query: &AnalyticsDashboardQuery) -> Vec<context::SortLinkView> {
    [
        ("Requests", "requests"),
        ("Cost", "cost"),
        ("Tokens", "tokens"),
        ("Last active", "last_active"),
    ]
    .into_iter()
    .map(|(label, key)| context::SortLinkView {
        label,
        href: urls::sort_url(query, key),
        is_active: LeaderboardSort::from_sort_param(query.sort.as_deref())
            == LeaderboardSort::from_sort_param(Some(key)),
    })
    .collect()
}

struct PageInput<'a> {
    query: &'a AnalyticsDashboardQuery,
    tab: DashboardTab,
    range: TimeRange,
    bucket: SeriesBucket,
    page: i64,
    filters: FiltersView,
    fetched: &'a data::AnalyticsDashboardData,
    slo_ms: i32,
}

fn page_context(input: PageInput<'_>) -> AnalyticsDashboardContext {
    let PageInput {
        query,
        tab,
        range,
        bucket,
        page,
        filters,
        fetched,
        slo_ms,
    } = input;
    let weekly = bucket == SeriesBucket::Week;

    let leaderboard = view_tables::leaderboard_view(fetched, &range, query, page);
    let chips = urls::active_chips(query);
    let has_active_filters = !chips.is_empty();
    let charts = view_models::overview_charts(fetched, query, &range, weekly);

    AnalyticsDashboardContext {
        page: "analytics-dashboard",
        title: "Analytics".to_owned(),
        time_range: view::time_range_view(query, &range),
        tabs: urls::tab_links(tab, query),
        is_overview: tab == DashboardTab::Overview,
        is_usage: tab == DashboardTab::Usage,
        is_spend: tab == DashboardTab::Spend,
        is_code: tab == DashboardTab::Code,

        filters,
        chips,
        has_active_filters,
        clear_url: urls::clear_url(query),
        base_url: BASE_URL,

        kpis: view::kpi_strip(
            &fetched.kpis,
            &range,
            (tab == DashboardTab::Overview).then_some(fetched.series.as_slice()),
        ),
        volume_chart: charts.volume,
        cost_chart: charts.cost,
        model_pie: charts.model_pie,
        model_cost_chart: charts.model_cost,

        leaderboard,
        permissions: view_tables::permission_stats(&fetched.permissions),

        slo_options: urls::slo_links(query, slo_ms),
        latency_link: "/admin/entities/requests".to_owned(),
        has_anomalies: !fetched.anomalies.is_empty(),
        anomalies: view_spend::anomaly_rows(&fetched.anomalies),
        fast_slow: view_spend::fast_slow(&fetched.latency),
        session_costs: view_spend::session_costs(&fetched.session_costs),

        commit_chart: view_code::commit_chart(&fetched.code_series, &range),
        loc_chart: view_code::loc_chart(&fetched.code_series, &range),
        code_frames: view_code::code_frames(&fetched.code_totals),
    }
}

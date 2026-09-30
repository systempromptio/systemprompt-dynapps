//! Data-collection orchestration for the site analytics dashboard.
//!
//! Runs only the queries the active tab renders; every failed `Result` goes
//! to the page's fault collector, so a single failed query degrades its own
//! panel and is named on the page rather than taking the page down.
//! The Overview tab is the exception and loads every site aggregate at once,
//! because its job is to put the whole instance on one screen — the five
//! entity tabs then each pay for their own queries alone.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::site::kpis::SiteKpis;
use crate::repositories::analytics::site::latency::LatencySplit;
use crate::repositories::analytics::site::leaderboards::{
    LeaderboardPage, LeaderboardSort, UserUsageRow,
};
use crate::repositories::analytics::site::model_series::ModelCostBucket;
use crate::repositories::analytics::site::series::{SeriesBucket, UsageBucket};
use crate::repositories::analytics::site::{
    SiteScope, distribution, kpis, latency, leaderboards, model_series, series,
};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::util::time_range::TimeRange;

use super::context::DashboardTab;
use super::data_tabs::{self, TabData, TabPlan};

#[derive(Default)]
pub(super) struct AnalyticsDashboardData {
    pub kpis: SiteKpis,
    pub series: Vec<UsageBucket>,
    pub models: Vec<distribution::ModelDistributionRow>,
    pub leaderboard: Vec<UserUsageRow>,
    pub leaderboard_total: i64,
    pub model_cost: Vec<ModelCostBucket>,
    pub latency: LatencySplit,
    pub tabs: TabData,
}

pub(super) struct DashboardQueryPlan<'a> {
    pub tab: DashboardTab,
    pub scope: &'a SiteScope,
    pub range: TimeRange,
    pub bucket: SeriesBucket,
    pub sort: LeaderboardSort,
    pub page_size: i64,
    pub offset: i64,
    pub slo_ms: i32,
    pub axis: crate::repositories::analytics::site::cost::ContainerAxis,
}

pub(super) async fn load_dashboard_data(
    pool: &Arc<PgPool>,
    plan: DashboardQueryPlan<'_>,
    faults: &mut Faults,
) -> AnalyticsDashboardData {
    let kpis = faults.take(
        "site kpis",
        kpis::get_site_kpis(pool, plan.range, plan.scope).await,
    );
    let mut data = AnalyticsDashboardData {
        kpis,
        ..AnalyticsDashboardData::default()
    };

    let tab_plan = TabPlan {
        scope: plan.scope,
        range: plan.range,
        limit: plan.page_size,
        offset: plan.offset,
        axis: plan.axis,
    };
    match plan.tab {
        DashboardTab::Overview => load_overview(pool, &plan, &mut data, faults).await,
        DashboardTab::Models => {
            data.tabs = data_tabs::load_models(pool, &tab_plan, faults).await;
        },
        DashboardTab::Skills => {
            data.tabs = data_tabs::load_skills(pool, &tab_plan, faults).await;
        },
        DashboardTab::Tools => {
            data.tabs = data_tabs::load_tools(pool, &tab_plan, faults).await;
        },
        DashboardTab::Sessions => {
            data.tabs = data_tabs::load_sessions(pool, &tab_plan, faults).await;
        },
        DashboardTab::Cost => {
            data.tabs = data_tabs::load_cost(pool, &tab_plan, faults).await;
        },
    }

    data
}

async fn load_overview(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
    faults: &mut Faults,
) {
    let (series_res, models_res, model_cost_res) = tokio::join!(
        series::list_daily_usage_series(pool, plan.range, plan.scope, plan.bucket),
        distribution::list_model_distribution(pool, plan.range, plan.scope),
        model_series::list_model_cost_series(pool, plan.range, plan.scope, plan.bucket),
    );
    data.series = faults.take("usage series", series_res);
    data.models = faults.take("model distribution", models_res);
    data.model_cost = faults.take("model cost series", model_cost_res);

    load_usage_tab(pool, plan, data, faults).await;
    load_spend_tab(pool, plan, data, faults).await;
}

async fn load_spend_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
    faults: &mut Faults,
) {
    data.latency = faults.take(
        "latency split",
        latency::get_latency_split(pool, plan.range, plan.scope, plan.slo_ms).await,
    );
}

async fn load_usage_tab(
    pool: &PgPool,
    plan: &DashboardQueryPlan<'_>,
    data: &mut AnalyticsDashboardData,
    faults: &mut Faults,
) {
    let leaders_res = leaderboards::list_top_users_by_requests(
        pool,
        plan.range,
        plan.scope,
        LeaderboardPage {
            sort: plan.sort,
            limit: plan.page_size,
            offset: plan.offset,
        },
    )
    .await;
    let (rows, total) = faults.take("top users", leaders_res);
    data.leaderboard = rows;
    data.leaderboard_total = total;
}

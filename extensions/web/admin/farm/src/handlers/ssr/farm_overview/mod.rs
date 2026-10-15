//! `/admin/farm` — every declared farm, what is running across them now, and
//! the headline figures. The KPI strip and the running board are the live
//! half ([`activity`]); the scheduled-flow strip ([`scheduled`]), the 30-day
//! outcome strip, the runs-over-time and success/eval trend charts and the
//! farm cards render once per page load.

pub(crate) mod activity;
pub(crate) mod cards;
pub(crate) mod scheduled;

use std::sync::Arc;

use axum::extract::State;
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use activity::{ActivityView, KpiTile, load_activity};
use cards::{FarmCardView, farm_card};
use chrono::{Duration, Utc};
use systemprompt_web_admin_core::ssr::types::svg_line::SvgLineChartView;

use super::sync_card::load_bundles;
use crate::handlers::ssr::shared::outcomes::{
    OutcomeTotals, outcome_kpis, rates_chart, runs_chart,
};
use crate::repositories::agents::list_all_agents;
use crate::repositories::clients::{ReportScope, list_daily_outcomes};
use crate::repositories::farms::list_farms;
use crate::repositories::reports::get_scheduled_stats;

const LIVE_URL: &str = "/admin/api/farm/activity";
const TREND_DAYS: i64 = 30;

#[derive(Debug, Serialize)]
struct FarmOverviewContext {
    page: &'static str,
    title: &'static str,
    subtitle: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    live_url: &'static str,
    activity: ActivityView,
    trend_kpis: Vec<KpiTile>,
    scheduled_kpis: Vec<KpiTile>,
    runs_chart: SvgLineChartView,
    rates_chart: SvgLineChartView,
    farms: Vec<FarmCardView>,
    farm_count: usize,
    has_farms: bool,
}

pub(crate) async fn farm_overview_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let mut faults = Faults::new();
    let trend_scope = ReportScope {
        client: None,
        user: None,
        since: Utc::now() - Duration::days(TREND_DAYS - 1),
    };
    let since = trend_scope.since;
    let (activity, (farms, agents), daily, scheduled) = tokio::join!(
        async {
            let mut live = Faults::new();
            let view = load_activity(&pool, &mut live).await;
            (view, live)
        },
        async { tokio::join!(list_farms(&pool), list_all_agents(&pool)) },
        list_daily_outcomes(&pool, trend_scope),
        get_scheduled_stats(&pool, since),
    );
    let scheduled = faults
        .take_or("scheduled runs", scheduled.map(Some), None)
        .map(|stats| scheduled::scheduled_kpis(&stats))
        .unwrap_or_default();
    let daily = faults.take("daily outcomes", daily);
    let window = format!("{TREND_DAYS} days");
    let (activity, live_faults) = activity;
    faults.extend(live_faults.into_vec());
    let farms = faults.take("farm list", farms);
    let agents = faults.take("agent list", agents);
    let bundles = faults.take("pinned bundles", load_bundles());
    let farms: Vec<FarmCardView> = farms
        .iter()
        .map(|f| farm_card(f, &agents, &bundles))
        .collect();

    let ctx = FarmOverviewContext {
        page: "farms",
        title: "Farms",
        subtitle: "Agent farms synced from a Git marketplace, what is running now, and how each farm last synced.",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current("Farms"),
        ],
        live_url: LIVE_URL,
        farm_count: farms.len(),
        has_farms: !farms.is_empty(),
        activity,
        scheduled_kpis: scheduled,
        trend_kpis: outcome_kpis(&OutcomeTotals::from_days(&daily), &daily, &window),
        runs_chart: runs_chart(&daily, &window),
        rates_chart: rates_chart(&daily, &window),
        farms,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-overview",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

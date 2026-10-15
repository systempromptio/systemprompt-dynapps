//! `/admin/farm/clients` and `/admin/farm/clients/{client_id}` — who the farm
//! runs are for. The list ranks every client by runs over the window with its
//! success rate, task completion, eval score and cost; one client adds the
//! KPI strip, the runs and success/eval trend charts, per-agent outcomes and
//! its recent runs with their completion reports. `?days=` picks 7, 30 or 90.

pub(crate) mod views;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::svg_line::SvgLineChartView;

use views::{
    AgentOutcomeView, ClientRowView, ReportRunView, agent_rows, client_rows, report_run_rows,
};

use crate::handlers::ssr::farm_overview::activity::KpiTile;
use crate::handlers::ssr::shared::outcomes::{
    OutcomeTotals, outcome_kpis, rates_chart, runs_chart,
};
use crate::repositories::clients::{
    ReportScope, find_client_summary, list_agent_outcomes, list_client_summaries,
    list_daily_outcomes, list_report_runs,
};

use crate::types::ClientId;

const RECENT_RUNS: i64 = 25;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct WindowQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct WindowOption {
    label: &'static str,
    href: String,
    is_active: bool,
}

const fn window_days(q: &WindowQuery) -> i64 {
    match q.days {
        Some(7) => 7,
        Some(90) => 90,
        _ => 30,
    }
}

fn window_tabs(base: &str, days: i64) -> Vec<WindowOption> {
    [(7, "7 days"), (30, "30 days"), (90, "90 days")]
        .into_iter()
        .map(|(d, label)| WindowOption {
            label,
            href: format!("{base}?days={d}"),
            is_active: d == days,
        })
        .collect()
}

fn scope(client: Option<&ClientId>, days: i64) -> ReportScope<'_> {
    ReportScope {
        client,
        user: None,
        since: Utc::now() - Duration::days(days - 1),
    }
}

#[derive(Debug, Serialize)]
struct ClientsPage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    window: String,
    tabs: Vec<WindowOption>,
    kpis: Vec<KpiTile>,
    runs_chart: SvgLineChartView,
    rates_chart: SvgLineChartView,
    clients: Vec<ClientRowView>,
    client_count: usize,
    has_clients: bool,
}

pub(crate) async fn clients_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(q): Query<WindowQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let days = window_days(&q);
    let mut faults = Faults::new();
    let (clients, daily) = tokio::join!(
        list_client_summaries(&pool, scope(None, days)),
        list_daily_outcomes(&pool, scope(None, days)),
    );
    let clients = faults.take("client summaries", clients);
    let daily = faults.take("daily outcomes", daily);
    let window = format!("{days} days");
    let mut totals = OutcomeTotals::from_days(&daily);
    totals.completed = Some((
        clients.iter().map(|c| c.completed).sum(),
        clients.iter().map(|c| c.reported).sum(),
    ));
    let rows = client_rows(&clients);
    let ctx = ClientsPage {
        page: "farm-clients",
        title: "Clients",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current("Clients"),
        ],
        tabs: window_tabs("/admin/farm/clients", days),
        kpis: outcome_kpis(&totals, &daily, &window),
        runs_chart: runs_chart(&daily, &window),
        rates_chart: rates_chart(&daily, &window),
        client_count: rows.len(),
        has_clients: !rows.is_empty(),
        clients: rows,
        window,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-clients",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

#[derive(Debug, Serialize)]
struct ClientPage {
    page: &'static str,
    title: String,
    subtitle: String,
    breadcrumbs: Vec<BreadcrumbView>,
    slug: String,
    odoo_partner_id: Option<i64>,
    window: String,
    tabs: Vec<WindowOption>,
    kpis: Vec<KpiTile>,
    runs_chart: SvgLineChartView,
    rates_chart: SvgLineChartView,
    agents: Vec<AgentOutcomeView>,
    has_agents: bool,
    runs: Vec<ReportRunView>,
    has_runs: bool,
    runs_url: String,
}

pub(crate) async fn client_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(client_id): Path<String>,
    Query(q): Query<WindowQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let days = window_days(&q);
    let client = ClientId::new(client_id);
    let id = &client;
    let summary = find_client_summary(&pool, scope(Some(id), days))
        .await
        .map_err(AdminError::internal)?
        .ok_or_else(|| AdminError::NotFound("No client with that id.".to_owned()))?;
    let mut faults = Faults::new();
    let (daily, agents, runs) = tokio::join!(
        list_daily_outcomes(&pool, scope(Some(id), days)),
        list_agent_outcomes(&pool, scope(Some(id), days)),
        list_report_runs(&pool, Some(id), None, RECENT_RUNS),
    );
    let daily = faults.take("daily outcomes", daily);
    let agents = agent_rows(&faults.take("agent outcomes", agents));
    let runs = report_run_rows(&faults.take("recent runs", runs));
    let window = format!("{days} days");
    let mut totals = OutcomeTotals::from_days(&daily);
    totals.eval_mean = summary.eval_mean;
    totals.completed = Some((summary.completed, summary.reported));
    let base = format!("/admin/farm/clients/{id}");
    let ctx = ClientPage {
        page: "farm-client",
        subtitle: format!(
            "Farm runs for {}: outcomes, task completion, eval scores and cost over the last {window}.",
            summary.name
        ),
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::link("Clients", "/admin/farm/clients"),
            BreadcrumbView::current(summary.name.clone()),
        ],
        title: summary.name.clone(),
        slug: summary.slug.clone(),
        odoo_partner_id: summary.odoo_partner_id,
        tabs: window_tabs(&base, days),
        kpis: outcome_kpis(&totals, &daily, &window),
        runs_chart: runs_chart(&daily, &window),
        rates_chart: rates_chart(&daily, &window),
        has_agents: !agents.is_empty(),
        agents,
        has_runs: !runs.is_empty(),
        runs,
        runs_url: "/admin/farm/runs".to_owned(),
        window,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-client",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

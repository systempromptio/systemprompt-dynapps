//! `GET /admin/farm/evals` — every eval suite, the latest verdict per case
//! pivoted against agents, how scores and pass rates move over the window,
//! and a Run now form per suite. Filterable by farm and window (`?days=`).

mod context;
pub(crate) mod matrix;
mod trends;

use std::sync::Arc;

use axum::extract::{Query, State};
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

use context::{EvalSuiteRowView, KpiView, OptionView, WINDOWS};
pub(crate) use context::{eval_status_badge, is_eval_live, score_display};
use matrix::{EvalMatrixView, eval_matrix};

use crate::repositories::evals::{
    list_all_eval_suites, list_eval_cases, list_eval_runs_paged, list_eval_suites,
    list_latest_results_matrix,
};
use crate::repositories::farms::list_farms;
use crate::types::FarmId;

const RUNS_SCAN: i64 = 500;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct EvalsQuery {
    pub farm: Option<String>,
    pub days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct EvalsPage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    days: u32,
    farm_id: Option<String>,
    farms: Vec<OptionView>,
    windows: Vec<OptionView>,
    kpis: Vec<KpiView>,
    matrix: EvalMatrixView,
    score_chart: SvgLineChartView,
    pass_chart: SvgLineChartView,
    suites: Vec<EvalSuiteRowView>,
    has_suites: bool,
}

pub(crate) async fn evals_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EvalsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let days = query
        .days
        .filter(|d| WINDOWS.contains(d))
        .unwrap_or(WINDOWS[1]);
    let farm_id = query
        .farm
        .filter(|f| !f.is_empty() && f.len() <= 128)
        .map(FarmId::new);
    let mut faults = Faults::new();
    let farms = faults.take("farms", list_farms(&pool).await);
    let suites = match &farm_id {
        Some(id) => faults.take("eval suites", list_eval_suites(&pool, id).await),
        None => faults.take("eval suites", list_all_eval_suites(&pool).await),
    };
    let mut cases = Vec::new();
    for suite in &suites {
        cases.extend(faults.take("eval cases", list_eval_cases(&pool, &suite.id).await));
    }
    let cells = faults.take(
        "eval matrix",
        list_latest_results_matrix(&pool, farm_id.as_ref()).await,
    );
    let (runs, _) = faults.take(
        "eval runs",
        list_eval_runs_paged(&pool, farm_id.as_ref(), None, RUNS_SCAN, 0).await,
    );
    let runs = context::in_window(runs, Utc::now() - Duration::days(i64::from(days)));

    let ctx = EvalsPage {
        page: "farm-evals",
        title: "Evals",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current("Evals"),
        ],
        days,
        farms: context::farm_options(&farms, farm_id.as_ref().map(FarmId::as_str)),
        farm_id: farm_id.map(|f| f.as_str().to_owned()),
        windows: context::window_options(days),
        kpis: context::kpis(&suites, &runs, days),
        matrix: eval_matrix(&suites, &cases, &cells),
        score_chart: trends::score_chart(&runs, days),
        pass_chart: trends::pass_rate_chart(&runs, days),
        suites: context::suite_views(&suites, &runs),
        has_suites: !suites.is_empty(),
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-evals",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

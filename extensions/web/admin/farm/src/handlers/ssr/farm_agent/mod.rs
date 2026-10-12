//! `/admin/farm/{farm}/agents/{agent}` — one agent: its definition, how to run
//! it remotely, its last thirty days, its recent runs and its eval cases.
//!
//! The launch dialog is a plain form posting to the launch action, so the
//! page works without script; `?launch=1` renders it open, which is also how
//! the action re-renders a refused submission beside its reason.

mod launch_form;
mod view;

use std::sync::Arc;

use axum::Extension;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use self::launch_form::{FormInputs, LaunchFormView, launch_form_view};
use self::view::{
    AgentKpiView, DefinitionView, HeaderView, MatrixView, definition_view, header_view,
    is_finished, kpi_views, matrix_view,
};
use super::curl::{RemoteCommands, remote_commands};
use super::shared::{RunRowView, run_row_views};
use crate::repositories::filters::RunFilter;
use crate::repositories::rows::{AgentRow, FarmRow};
use crate::repositories::{agents, evals, farms, fixtures, runs};
use crate::services::farm_sync::eval_suite_id;
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId};

pub(crate) use launch_form::{LaunchValues, PROMPT_MAX, TIMEOUT_RANGE, TURNS_RANGE, model_choices};

const RECENT_RUNS: i64 = 20;
const KPI_WINDOW_DAYS: i64 = 30;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct AgentQuery {
    pub launch: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentPageContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    header: HeaderView,
    kpis: Vec<AgentKpiView>,
    definition: DefinitionView,
    remote: RemoteCommands,
    devices_url: &'static str,
    open_launch_url: String,
    runs: Vec<RunRowView>,
    matrix: Option<MatrixView>,
    launch: LaunchFormView,
}

pub(crate) fn require_console(shell: &Page) -> Result<(), AdminError> {
    if shell.user.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

pub(crate) async fn load_agent(
    state: &FarmState,
    farm_id: &FarmId,
    agent_id: &FarmAgentId,
) -> Result<Option<(FarmRow, AgentRow)>, AdminError> {
    let Some(farm) = farms::find_farm(&state.pool, farm_id).await? else {
        return Ok(None);
    };
    let agent = agents::find_agent(&state.pool, farm_id, agent_id).await?;
    Ok(agent.map(|a| (farm, a)))
}

pub(crate) async fn farm_agent_page(
    shell: Page,
    Extension(state): Extension<Arc<FarmState>>,
    Path((farm_id, agent_id)): Path<(String, String)>,
    Query(query): Query<AgentQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let (farm_id, agent_id) = (FarmId::new(farm_id), FarmAgentId::new(agent_id));
    let (farm, agent) = load_agent(&state, &farm_id, &agent_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("agent {}", agent_id.as_str())))?;
    let values = LaunchValues::defaults(&farm, &agent);
    let open = query.launch.is_some_and(|v| v != "0");
    render_agent_page(
        &shell,
        &state,
        &farm,
        &agent,
        Launch {
            values,
            open,
            error: None,
        },
    )
    .await
}

pub(crate) struct Launch {
    pub values: LaunchValues,
    pub open: bool,
    pub error: Option<String>,
}

pub(crate) async fn render_agent_page(
    shell: &Page,
    state: &FarmState,
    farm: &FarmRow,
    agent: &AgentRow,
    launch: Launch,
) -> AdminHtmlResult<Response> {
    let pool = &state.pool;
    let mut faults = Faults::new();
    let filter = RunFilter {
        farm_id: Some(farm.id.clone()),
        agent_id: Some(agent.id.clone()),
        since: Some(Utc::now() - Duration::days(KPI_WINDOW_DAYS)),
        ..RunFilter::default()
    };
    let run_stats = runs::get_run_stats(pool, &filter).await?;
    let recent = runs::list_recent_runs(pool, Some(&farm.id), Some(&agent.id), RECENT_RUNS).await?;
    let suite_id = eval_suite_id(&farm.id, agent.id.as_str());
    let cases = faults.take("eval cases", evals::list_eval_cases(pool, &suite_id).await);
    let cells: Vec<_> = faults
        .take(
            "eval results",
            evals::list_latest_results_matrix(pool, Some(&farm.id)).await,
        )
        .into_iter()
        .filter(|c| c.agent_id == agent.id)
        .collect();
    let fixture_rows = faults.take("fixtures", fixtures::list_fixtures(pool, &farm.id).await);
    let finished: Vec<_> = recent.iter().filter(|r| is_finished(r)).cloned().collect();
    let page_url = format!(
        "/admin/farm/{}/agents/{}",
        farm.id.as_str(),
        agent.id.as_str()
    );
    let refused = launch.error.is_some();
    let ctx = AgentPageContext {
        page: "farm-agent",
        title: agent.name.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::link(
                farm.name.clone(),
                format!("/admin/farm/{}", farm.id.as_str()),
            ),
            BreadcrumbView::current(agent.name.clone()),
        ],
        header: header_view(farm, agent),
        kpis: kpi_views(&run_stats, &finished, &cells),
        definition: definition_view(agent),
        remote: remote_commands(
            &state.gateway.external_base,
            &farm.id,
            &agent.id,
            &agent.entry_prompt,
        ),
        devices_url: "/admin/devices",
        open_launch_url: format!("{page_url}?launch=1"),
        runs: run_row_views(&recent),
        matrix: matrix_view(farm, suite_id.as_str(), &cases, &cells),
        launch: launch_form_view(&FormInputs {
            farm,
            agent,
            fixtures: &fixture_rows,
            values: &launch.values,
            open: launch.open,
            error: launch.error,
        }),
    };
    let page = render_page_with_faults(
        &shell.engine,
        "farm-agent",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    );
    if refused {
        return Ok((StatusCode::UNPROCESSABLE_ENTITY, page).into_response());
    }
    Ok(page)
}

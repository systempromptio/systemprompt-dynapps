//! `/admin/farm/runs/{run_id}` — one farm run, live, on whichever host ran it.
//!
//! The page is a header, a KPI strip and six client tabs (Timeline,
//! Transcript, Report, Governance, Cost, Artifacts). While the run is live the
//! page polls `/admin/api/farm/runs/{run_id}?after=<seq>` (see
//! [`crate::handlers::ssr::fragments::run`]), which re-renders the same
//! `farm/run-live` block from the same [`RunData`], so a number never reads
//! differently after a poll than it did on first load.
//!
//! [`RunData`] pairs the farm row with the executor's `agent_runs` row. Its
//! event and tool-call counts are the counters stamped on the farm row once
//! the run finished, and a live count from `agent_run_events` before that.
//! `runner` is the Fly Machine the run executed on, when it ran on one.
//!
//! - [`context`] — header, identifiers, actions and the live block.
//! - [`summary`] — the KPI strip.
//! - [`cost`] — the request ledger and per-turn chart.
//! - [`artifacts`] — the workspace diff, exit state and structured output.
//! - [`report`] — the task-completion report and the identity badges.
//! - [`governance`] — the governance decisions the run's sessions produced.

pub(crate) mod artifacts;
pub(crate) mod context;
pub(crate) mod cost;
pub(crate) mod governance;
pub(crate) mod report;
pub(crate) mod summary;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, AgentRunRow};
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::handlers::ssr::run_events_view::{RunEventView, event_views};
use crate::handlers::ssr::shared::is_live;
use crate::repositories::clients::{RunDecisionRow, RunIdentityRow, RunReportRow};
use crate::repositories::rows::RunRow;
use crate::repositories::runs::RunnerMachineRow;
use crate::repositories::summaries::RunRequestRow;
use crate::repositories::{agents, clients, runs};
use crate::types::FarmRunId;

const EVENT_READ_CAP: i64 = 5_000;
const GOVERNANCE_CAP: i64 = 500;

#[derive(Debug)]
pub(crate) struct RunData {
    pub run: RunRow,
    pub agent_run: AgentRunRow,
    pub agent_name: Option<String>,
    pub context_id: Option<ContextId>,
    pub events: Vec<RunEventView>,
    pub event_count: i64,
    pub tool_call_count: i64,
    pub requests: Vec<RunRequestRow>,
    pub runner: Option<RunnerMachineRow>,
    pub report: Option<RunReportRow>,
    pub identity: Option<RunIdentityRow>,
    pub decisions: Vec<RunDecisionRow>,
}

pub(crate) fn not_found() -> AdminError {
    AdminError::NotFound("No farm run with that id.".to_owned())
}

pub(crate) async fn load_run_data(
    pool: &Arc<PgPool>,
    run_id: &FarmRunId,
    faults: &mut Faults,
) -> AdminResult<Option<RunData>> {
    let Some((run, agent_run)) = runs::find_run_with_agent_run(pool, run_id)
        .await
        .map_err(AdminError::internal)?
    else {
        return Ok(None);
    };
    let repo = AgentRunRepository::from_pool(Arc::clone(pool));
    let rows = repo
        .list_events_after(&run.agent_run_id, 0, EVENT_READ_CAP)
        .await
        .map_err(AdminError::internal)?;
    let events = event_views(&rows, agent_run.started_at.or(run.started_at));
    let (event_count, tool_call_count) = if is_live(&run.status) {
        let live_tools =
            i64::try_from(events.iter().filter(|e| e.is_tool_use).count()).unwrap_or(i64::MAX);
        let count = repo
            .count_events(&run.agent_run_id)
            .await
            .map_err(AdminError::internal)?;
        (count, live_tools)
    } else {
        (i64::from(run.event_count), i64::from(run.tool_call_count))
    };
    let agent_name = faults
        .take_or(
            "farm agent",
            agents::find_agent(pool, &run.farm_id, &run.agent_id).await,
            None,
        )
        .map(|a| a.name);
    let context_id = match (&run.context_id, &agent_run.host_session_id) {
        (Some(c), _) => Some(c.clone()),
        (None, Some(host_session)) => faults.take_or(
            "gateway conversation",
            runs::find_run_context_id(pool, &run.user_id, host_session).await,
            None,
        ),
        (None, None) => None,
    };
    let requests = match &agent_run.gateway_session_id {
        Some(session) => faults.take(
            "gateway requests",
            runs::list_run_requests(pool, session).await,
        ),
        None => Vec::new(),
    };
    let runner = faults.take_or(
        "runner machine",
        runs::find_run_runner(pool, &run.id).await,
        None,
    );
    let (report, identity, decisions) =
        load_report_facts(pool, &run, &agent_run, context_id.as_ref()).await;
    let report = faults.take_or("completion report", report, None);
    let identity = faults.take_or("run identity", identity, None);
    let decisions = faults.take("governance decisions", decisions);
    Ok(Some(RunData {
        run,
        agent_run,
        agent_name,
        context_id,
        events,
        event_count,
        tool_call_count,
        requests,
        runner,
        report,
        identity,
        decisions,
    }))
}

type ReportFacts = (
    Result<Option<RunReportRow>, sqlx::Error>,
    Result<Option<RunIdentityRow>, sqlx::Error>,
    Result<Vec<RunDecisionRow>, sqlx::Error>,
);

async fn load_report_facts(
    pool: &PgPool,
    run: &RunRow,
    agent_run: &AgentRunRow,
    context_id: Option<&ContextId>,
) -> ReportFacts {
    let sessions: Vec<String> = [
        agent_run
            .gateway_session_id
            .as_ref()
            .map(|s| s.as_str().to_owned()),
        agent_run
            .host_session_id
            .as_ref()
            .map(|s| s.as_str().to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect();
    let id = &run.id;
    tokio::join!(
        clients::find_run_report(pool, id),
        clients::find_run_identity(pool, id),
        clients::list_run_governance(pool, &sessions, context_id, GOVERNANCE_CAP),
    )
}

pub(crate) async fn run_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(run_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let mut faults = Faults::new();
    let Some(data) = load_run_data(&pool, &FarmRunId::new(run_id), &mut faults).await? else {
        return Err(not_found().into());
    };
    let ctx = context::detail_context(&data);
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-run-detail",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

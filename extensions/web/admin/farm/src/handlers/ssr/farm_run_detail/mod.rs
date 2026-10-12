//! `/admin/farm/runs/{run_id}` — one headless Claude Code run, live.
//!
//! The page is a header, a KPI strip and four client tabs (Timeline,
//! Transcript, Cost, Artifacts). While the run is live the page polls
//! `/admin/api/farm/runs/{run_id}?after=<seq>` (see
//! [`crate::handlers::ssr::fragments::run`]), which re-renders the same
//! `farm/run-live` block from the same [`RunData`], so a number never reads
//! differently after a poll than it did on first load.
//!
//! - [`context`] — header, identifiers, actions and the live block.
//! - [`summary`] — the KPI strip.
//! - [`cost`] — the request ledger and per-turn chart.
//! - [`artifacts`] — the workspace diff and exit state.

pub(crate) mod artifacts;
pub(crate) mod context;
pub(crate) mod cost;
pub(crate) mod summary;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::handlers::ssr::run_events_view::{RunEventView, event_views};
use crate::repositories::rows::RunRow;
use crate::repositories::summaries::RunRequestRow;
use crate::repositories::{agents, run_events, runs};
use crate::types::FarmRunId;

#[derive(Debug)]
pub(crate) struct RunData {
    pub run: RunRow,
    pub agent_name: Option<String>,
    pub context_id: Option<ContextId>,
    pub events: Vec<RunEventView>,
    pub requests: Vec<RunRequestRow>,
}

pub(crate) fn not_found() -> AdminError {
    AdminError::NotFound("No farm run with that id.".to_owned())
}

pub(crate) async fn load_run_data(
    pool: &PgPool,
    run_id: &FarmRunId,
    faults: &mut Faults,
) -> AdminResult<Option<RunData>> {
    let Some(run) = runs::find_run(pool, run_id).await? else {
        return Ok(None);
    };
    let rows = run_events::list_run_events(pool, run_id).await?;
    let events = event_views(&rows, run.started_at);
    let agent_name = faults
        .take_or(
            "farm agent",
            agents::find_agent(pool, &run.farm_id, &run.agent_id).await,
            None,
        )
        .map(|a| a.name);
    let context_id = match (&run.context_id, &run.claude_session_id) {
        (Some(c), _) => Some(c.clone()),
        (None, Some(claude)) => faults.take_or(
            "gateway conversation",
            runs::find_run_context_id(pool, &run.user_id, claude).await,
            None,
        ),
        (None, None) => None,
    };
    let requests = match &run.gateway_session_id {
        Some(session) => faults.take(
            "gateway requests",
            runs::list_run_requests(pool, session).await,
        ),
        None => Vec::new(),
    };
    Ok(Some(RunData {
        run,
        agent_name,
        context_id,
        events,
        requests,
    }))
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

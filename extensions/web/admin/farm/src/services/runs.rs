//! Launching and cancelling runs: the one entry point the dashboard, the
//! public API and the eval service share.
//!
//! A launch resolves the agent's and farm's defaults, records the run
//! `queued` and hands its id to the runner.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;

use super::runner::outcome;
use super::runner::prepare::render_entry_prompt;
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::NewRun;
use crate::repositories::rows::RunRow;
use crate::repositories::{agents, farms, runs};
use crate::state::FarmState;
use crate::types::{EvalResultId, FarmAgentId, FarmId, FarmRunId, RunStatus, TriggeredVia};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub user_id: UserId,
    pub triggered_via: TriggeredVia,
    pub prompt: Option<String>,
    pub case_prompt: Option<String>,
    pub fixture: Option<String>,
    pub model: Option<String>,
    pub max_turns: Option<i32>,
    pub max_budget_usd: Option<f64>,
    pub timeout_seconds: Option<i32>,
    pub eval_result_id: Option<EvalResultId>,
}

pub async fn launch(state: &Arc<FarmState>, req: LaunchRequest) -> FarmResult<RunRow> {
    let runner = state
        .runner()
        .cloned()
        .ok_or_else(|| FarmError::Config("the agent farm runner is not running".to_owned()))?;
    let pool = state.pool.as_ref();
    let farm = farms::find_farm(pool, &req.farm_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("farm {}", req.farm_id)))?;
    let agent = agents::find_agent(pool, &req.farm_id, &req.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}/{}", req.farm_id, req.agent_id)))?;

    let template = req
        .prompt
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| agent.entry_prompt.clone());
    let case_prompt = req.case_prompt.unwrap_or_default();
    let prompt = render_entry_prompt(&template, "{{fixture_path}}", &case_prompt);
    if prompt.trim().is_empty() {
        return Err(FarmError::Validation(
            "the run has no prompt and the agent declares no entry prompt".to_owned(),
        ));
    }

    let limits = state.limits;
    let max_turns = req
        .max_turns
        .or(agent.max_turns)
        .or_else(|| (farm.max_turns > 0).then_some(farm.max_turns))
        .unwrap_or(limits.default_max_turns as i32);
    let max_budget_usd = req
        .max_budget_usd
        .or(agent.max_budget_usd)
        .or_else(|| (farm.max_budget_usd > 0.0).then_some(farm.max_budget_usd))
        .unwrap_or(limits.default_max_budget_usd);
    let timeout_seconds = req
        .timeout_seconds
        .or_else(|| (farm.timeout_seconds > 0).then_some(farm.timeout_seconds))
        .unwrap_or(limits.default_timeout_seconds as i32);
    if max_turns < 1 || max_budget_usd <= 0.0 || timeout_seconds < 1 {
        return Err(FarmError::Validation(
            "max_turns, max_budget_usd and timeout_seconds must be positive".to_owned(),
        ));
    }

    let new_run = NewRun {
        id: FarmRunId::new(uuid::Uuid::new_v4().to_string()),
        farm_id: req.farm_id,
        agent_id: req.agent_id,
        user_id: req.user_id,
        triggered_via: req.triggered_via,
        prompt,
        fixture: req.fixture.or_else(|| agent.fixture.clone()),
        model: req.model.or_else(|| agent.model.clone()).or(farm.model),
        permission_mode: farm.permission_mode,
        max_turns,
        max_budget_usd,
        timeout_seconds,
        source_commit_sha: farm.synced_commit_sha,
        eval_result_id: req.eval_result_id,
    };
    let row = runs::insert_run(pool, &new_run).await?;
    runner.enqueue(row.id.clone()).await?;
    Ok(row)
}

pub async fn cancel(
    state: &Arc<FarmState>,
    run_id: &FarmRunId,
    user_id: &UserId,
) -> FarmResult<bool> {
    let pool = state.pool.as_ref();
    let run = runs::find_run(pool, run_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("run {run_id}")))?;
    let status = RunStatus::parse(&run.status);
    if status.is_none_or(RunStatus::is_terminal) {
        return Ok(false);
    }
    tracing::info!(run = %run_id, actor = %user_id, "farm run cancel requested");
    if status == Some(RunStatus::Queued) {
        let cancelled = outcome::terminal(
            RunStatus::Cancelled,
            Some(format!("cancelled by {user_id}")),
        );
        runs::update_run_finished(pool, run_id, &cancelled).await?;
        return Ok(true);
    }
    let flagged = runs::update_run_cancel_requested(pool, run_id).await?;
    Ok(flagged
        && state
            .runner()
            .is_some_and(|runner| runner.request_cancel(run_id)))
}

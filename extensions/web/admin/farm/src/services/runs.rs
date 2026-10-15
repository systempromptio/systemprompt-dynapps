//! Launching and cancelling runs: the one entry point the dashboard, the
//! public API and the eval service share.
//!
//! A launch resolves the agent's and farm's defaults, builds core's
//! `RunRequest`, and in one transaction queues the `agent_runs` row and the
//! `agent_farm_runs` row that references it. The farm's runner service
//! claims it from there. A cancel stamps both rows; a run still queued is
//! finished at once, a running one is stopped by its handler.

use std::sync::Arc;

use sqlx::PgPool;

use serde::{Deserialize, Serialize};
use systemprompt::agent_runs::AgentRunRepository;
use systemprompt::agent_runs::repository::NewAgentRun;
use systemprompt::identifiers::{AgentRunId, UserId};

use super::runner::{ORIGIN_KIND, observer, prepare};
use crate::error::{FarmError, FarmResult};
use crate::repositories::filters::NewRun;
use crate::repositories::rows::{AgentRow, FarmRow, RunRow};
use crate::repositories::{agents, clients, farms, runs};
use crate::state::{FarmState, RunnerLimits};
use crate::types::{
    ClientId, EvalResultId, FarmAgentId, FarmId, FarmRunId, RunStatus, TriggeredVia,
};

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
    pub client_id: Option<ClientId>,
}

pub async fn launch(state: &Arc<FarmState>, req: LaunchRequest) -> FarmResult<RunRow> {
    let pool = state.pool.as_ref();
    let farm = farms::find_farm(pool, &req.farm_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("farm {}", req.farm_id)))?;
    let agent = agents::find_agent(pool, &req.farm_id, &req.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}/{}", req.farm_id, req.agent_id)))?;

    let (max_turns, max_budget_usd, timeout_seconds) =
        resolve_limits(&req, &agent, &farm, state.limits)?;
    let template = req
        .prompt
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| agent.entry_prompt.clone());
    let model = req.model.or_else(|| agent.model.clone()).or(farm.model);

    let farm_run_id = FarmRunId::new(uuid::Uuid::new_v4().to_string());
    let agent_run_id = AgentRunId::generate();
    let layout = prepare::run_layout(state, &agent_run_id);
    let request = prepare::build_request(prepare::RequestSpec {
        agent: &agent,
        farm_id: &req.farm_id,
        farm_run_id: &farm_run_id,
        layout: &layout,
        mcp_base: &state.gateway.internal_base,
        prompt_template: &template,
        case_prompt: req.case_prompt.as_deref(),
        model: model.as_deref(),
        permission_mode: &farm.permission_mode,
        max_turns,
        max_budget_usd,
        timeout_seconds,
    })?;
    if request.prompt.trim().is_empty() {
        return Err(FarmError::Validation(
            "the run has no prompt and the agent declares no entry prompt".to_owned(),
        ));
    }

    let credential_user = prepare::credential_user(
        agent.run_as_user_id.as_ref(),
        farm.run_as_user_id.as_ref(),
        &req.user_id,
    );
    let new_run = NewRun {
        id: farm_run_id,
        agent_run_id,
        farm_id: req.farm_id,
        agent_id: req.agent_id,
        user_id: req.user_id,
        triggered_via: req.triggered_via,
        host: request.host.as_str().to_owned(),
        attempt: i32::try_from(request.attempt).unwrap_or(1),
        prompt: request.prompt.clone(),
        fixture: req.fixture.or_else(|| agent.fixture.clone()),
        model,
        permission_mode: farm.permission_mode,
        max_turns,
        max_budget_usd,
        timeout_seconds,
        source_commit_sha: farm.synced_commit_sha,
        eval_result_id: req.eval_result_id,
        client_id: req.client_id,
    };

    insert_pair(pool, &new_run, &credential_user, &request).await
}

pub async fn resolve_client(pool: &PgPool, slug: Option<&str>) -> FarmResult<Option<ClientId>> {
    let Some(slug) = slug.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    clients::find_client(pool, slug)
        .await?
        .map(|row| Some(row.id))
        .ok_or_else(|| FarmError::Validation(format!("client {slug:?} is not declared")))
}

fn resolve_limits(
    req: &LaunchRequest,
    agent: &AgentRow,
    farm: &FarmRow,
    limits: RunnerLimits,
) -> FarmResult<(i32, f64, i32)> {
    let max_turns = req
        .max_turns
        .or(agent.max_turns)
        .or_else(|| (farm.max_turns > 0).then_some(farm.max_turns))
        .unwrap_or_else(|| i32::try_from(limits.default_max_turns).unwrap_or(i32::MAX));
    let max_budget_usd = req
        .max_budget_usd
        .or(agent.max_budget_usd)
        .or_else(|| (farm.max_budget_usd > 0.0).then_some(farm.max_budget_usd))
        .unwrap_or(limits.default_max_budget_usd);
    let timeout_seconds = req
        .timeout_seconds
        .or_else(|| (farm.timeout_seconds > 0).then_some(farm.timeout_seconds))
        .unwrap_or_else(|| i32::try_from(limits.default_timeout_seconds).unwrap_or(i32::MAX));
    if max_budget_usd <= 0.0 {
        return Err(FarmError::Validation(
            "max_budget_usd must be positive".to_owned(),
        ));
    }
    Ok((max_turns, max_budget_usd, timeout_seconds))
}

async fn insert_pair(
    pool: &PgPool,
    new_run: &NewRun,
    credential_user: &UserId,
    request: &systemprompt::agent_runs::RunRequest,
) -> FarmResult<RunRow> {
    let new_agent_run = NewAgentRun {
        id: &new_run.agent_run_id,
        origin_kind: ORIGIN_KIND,
        origin_id: new_run.id.as_str(),
        user_id: credential_user,
        request,
        not_before: None,
        task_id: None,
        context_id: None,
    };
    let mut tx = pool.begin().await?;
    AgentRunRepository::insert_run_in_tx(&mut tx, new_agent_run).await?;
    let row = runs::insert_run(&mut *tx, new_run).await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn cancel(state: &FarmState, run_id: &FarmRunId, user_id: &UserId) -> FarmResult<bool> {
    let pool = state.pool.as_ref();
    let run = runs::find_run(pool, run_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("run {run_id}")))?;
    let status = RunStatus::parse(&run.status);
    if status.is_none_or(RunStatus::is_terminal) {
        return Ok(false);
    }
    tracing::info!(run = %run_id, actor = %user_id, "farm run cancel requested");
    let repo = AgentRunRepository::from_pool(Arc::clone(&state.pool));
    let flagged = runs::update_run_cancel_requested(pool, run_id).await?;
    if repo.cancel_queued(&run.agent_run_id).await? {
        observer::observe_finished(state, &run.agent_run_id).await?;
    } else {
        repo.update_cancel_requested(&run.agent_run_id).await?;
    }
    Ok(flagged)
}

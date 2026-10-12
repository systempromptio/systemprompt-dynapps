//! `POST /runs`: launch one agent run as the caller, optionally from an eval
//! case's prompt and fixture, and optionally wait for it to finish.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::run_json::run_json;
use super::shared::{load_farm, triggered_via};
use crate::repositories::rows::{EvalCaseRow, RunRow};
use crate::repositories::{agents, evals, runs};
use crate::services::entitlement::require_farm_access;
use crate::services::farm_sync::eval_suite_id;
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId, RunStatus};

const WAIT_POLL: Duration = Duration::from_secs(2);
const WAIT_LIMIT: Duration = Duration::from_secs(600);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaunchBody {
    #[serde(alias = "farm_id")]
    farm: FarmId,
    #[serde(alias = "agent_id")]
    agent: FarmAgentId,
    prompt: Option<String>,
    case: Option<String>,
    fixture: Option<String>,
    model: Option<String>,
    max_turns: Option<i32>,
    max_budget_usd: Option<f64>,
    timeout_seconds: Option<i32>,
    #[serde(default)]
    wait: bool,
}

fn check_limits(body: &LaunchBody) -> AdminResult<()> {
    let bad_budget = body
        .max_budget_usd
        .is_some_and(|b| !b.is_finite() || b <= 0.0);
    if bad_budget {
        return Err(AdminError::Unprocessable(
            "max_budget_usd must be a positive number".to_owned(),
        ));
    }
    let non_positive = |v: Option<i32>| v.is_some_and(|v| v <= 0);
    if non_positive(body.max_turns) || non_positive(body.timeout_seconds) {
        return Err(AdminError::Unprocessable(
            "max_turns and timeout_seconds must be positive".to_owned(),
        ));
    }
    Ok(())
}

async fn find_case(
    state: &FarmState,
    farm: &FarmId,
    agent: &FarmAgentId,
    case_id: &str,
) -> AdminResult<EvalCaseRow> {
    let suite = eval_suite_id(farm, agent.as_str());
    evals::list_eval_cases(&state.pool, &suite)
        .await?
        .into_iter()
        .find(|c| c.id == case_id)
        .ok_or_else(|| {
            AdminError::NotFound(format!("eval case {case_id} for agent {}", agent.as_str()))
        })
}

pub(crate) async fn launch_run(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    Json(body): Json<LaunchBody>,
) -> AdminResult<Response> {
    check_limits(&body)?;
    let farm = load_farm(&state.pool, &body.farm).await?;
    require_farm_access(&state.pool, &ctx, &farm).await?;
    agents::find_agent(&state.pool, &body.farm, &body.agent)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("agent {}", body.agent.as_str())))?;
    let case = match body.case.as_deref() {
        Some(case_id) => Some(find_case(&state, &body.farm, &body.agent, case_id).await?),
        None => None,
    };
    let (case_prompt, case_fixture) = case.map_or((None, None), |c| (c.prompt, c.fixture));
    let request = LaunchRequest {
        farm_id: body.farm,
        agent_id: body.agent,
        user_id: ctx.user_id.clone(),
        triggered_via: triggered_via(&ctx),
        prompt: body.prompt,
        case_prompt,
        fixture: body.fixture.or(case_fixture),
        model: body.model,
        max_turns: body.max_turns,
        max_budget_usd: body.max_budget_usd,
        timeout_seconds: body.timeout_seconds,
        eval_result_id: None,
    };
    let run = launch(&state, request).await?;
    if !body.wait {
        return accepted(&state, run).await;
    }
    let run = wait_for_terminal(&state, run).await?;
    let finished = RunStatus::parse(&run.status).is_some_and(RunStatus::is_terminal);
    if finished {
        Ok(Json(run_json(&state.pool, run).await?).into_response())
    } else {
        accepted(&state, run).await
    }
}

async fn accepted(state: &FarmState, run: RunRow) -> AdminResult<Response> {
    Ok((
        StatusCode::ACCEPTED,
        Json(run_json(&state.pool, run).await?),
    )
        .into_response())
}

async fn wait_for_terminal(state: &FarmState, mut run: RunRow) -> AdminResult<RunRow> {
    let deadline = tokio::time::Instant::now() + WAIT_LIMIT;
    while tokio::time::Instant::now() < deadline {
        if RunStatus::parse(&run.status).is_some_and(RunStatus::is_terminal) {
            break;
        }
        tokio::time::sleep(WAIT_POLL).await;
        if let Some(latest) = runs::find_run(&state.pool, &run.id).await? {
            run = latest;
        }
    }
    Ok(run)
}

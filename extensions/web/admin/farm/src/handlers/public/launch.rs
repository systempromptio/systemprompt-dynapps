//! `POST /runs`: launch one agent run as the caller and optionally wait for
//! it to finish. Naming a `case` makes the launch an eval of that case: a
//! one-case eval run is recorded beside the run, and its assertions and judge
//! run when the run finishes; the response carries `eval_run_id`.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use systemprompt::ai::AiService;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use super::run_json::{RunJson, run_json};
use super::shared::{load_farm, triggered_via};
use crate::repositories::rows::RunRow;
use crate::repositories::{agents, runs};
use crate::services::entitlement::require_farm_access;
use crate::services::evals::{CaseLaunch, start_case_eval};
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{EvalRunId, FarmAgentId, FarmId, RunStatus};

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

#[derive(Debug, Serialize)]
struct LaunchedJson {
    #[serde(flatten)]
    run: RunJson,
    eval_run_id: Option<EvalRunId>,
}

pub(crate) async fn launch_run(
    State(state): State<Arc<FarmState>>,
    Extension(ctx): Extension<UserContext>,
    ai: Option<Extension<Option<Arc<AiService>>>>,
    Json(body): Json<LaunchBody>,
) -> AdminResult<Response> {
    check_limits(&body)?;
    let farm = load_farm(&state.pool, &body.farm).await?;
    require_farm_access(&state.pool, &ctx, &farm).await?;
    agents::find_agent(&state.pool, &body.farm, &body.agent)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("agent {}", body.agent.as_str())))?;
    let request = LaunchRequest {
        farm_id: body.farm,
        agent_id: body.agent,
        user_id: ctx.user_id.clone(),
        triggered_via: triggered_via(&ctx),
        prompt: body.prompt,
        case_prompt: None,
        fixture: body.fixture,
        model: body.model,
        max_turns: body.max_turns,
        max_budget_usd: body.max_budget_usd,
        timeout_seconds: body.timeout_seconds,
        eval_result_id: None,
    };
    let (run, eval_run_id) = match body.case.filter(|c| !c.trim().is_empty()) {
        Some(case_id) => {
            let request_ai = ai.and_then(|Extension(ai)| ai);
            let judge = state.judge.judge_for(request_ai, &ctx.user_id);
            let launched =
                start_case_eval(Arc::clone(&state), judge, CaseLaunch { request, case_id }).await?;
            (launched.run, Some(launched.eval_run.id))
        },
        None => (launch(&state, request).await?, None),
    };
    if !body.wait {
        return accepted(&state, run, eval_run_id).await;
    }
    let run = wait_for_terminal(&state, run).await?;
    let finished = RunStatus::parse(&run.status).is_some_and(RunStatus::is_terminal);
    if finished {
        let run = run_json(&state.pool, run).await?;
        Ok(Json(LaunchedJson { run, eval_run_id }).into_response())
    } else {
        accepted(&state, run, eval_run_id).await
    }
}

async fn accepted(
    state: &FarmState,
    run: RunRow,
    eval_run_id: Option<EvalRunId>,
) -> AdminResult<Response> {
    let run = run_json(&state.pool, run).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(LaunchedJson { run, eval_run_id }),
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

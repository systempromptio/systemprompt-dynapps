//! The trigger beyond a plain suite launch: one eval case run on the caller's
//! own launch (the public API's and MCP's `case`).
//!
//! A case trigger records a one-case eval run and its pending result, then
//! launches the farm run itself, linked to the result by `eval_result_id`, so
//! the caller gets the run back at once with the caller's own prompt, model
//! and limits. A background task waits for the run, checks the case's
//! assertions, asks the judge, and closes the eval run.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use super::case::{CaseJob, settle_case};
use super::judge::EvalJudge;
use super::orchestrator::{Tally, close_eval_run, fail_eval_run, record_outcome, set_cancelled};
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{AgentRow, EvalCaseRow, EvalRunRow, RunRow};
use crate::repositories::{agents, evals};
use crate::services::farm_sync::eval_suite_id;
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{EvalResultId, EvalRunId, EvalRunStatus, FarmId};

/// A launch that is also an eval of one case: `request` carries the caller's
/// overrides and trigger; the case supplies its prompt, fixture and timeout
/// where the request leaves them unset.
#[derive(Debug, Clone)]
pub struct CaseLaunch {
    pub request: LaunchRequest,
    pub case_id: String,
}

/// What a case trigger started: the one-case eval run and the farm run.
#[derive(Debug, Clone)]
pub struct CaseLaunched {
    pub eval_run: EvalRunRow,
    pub run: RunRow,
}

pub async fn start_case_eval(
    state: Arc<FarmState>,
    judge: Option<Arc<dyn EvalJudge>>,
    launch_case: CaseLaunch,
) -> FarmResult<CaseLaunched> {
    let CaseLaunch {
        mut request,
        case_id,
    } = launch_case;
    let pool = &state.pool;
    let suite_id = eval_suite_id(&request.farm_id, request.agent_id.as_str());
    let agent = agents::find_agent(pool, &request.farm_id, &request.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}", request.agent_id.as_str())))?;
    let case = evals::list_eval_cases(pool, &suite_id)
        .await?
        .into_iter()
        .find(|c| c.id == case_id)
        .ok_or_else(|| {
            FarmError::NotFound(format!(
                "eval case {case_id} for agent {}",
                request.agent_id.as_str()
            ))
        })?;
    let eval_run_id = EvalRunId::new(uuid::Uuid::new_v4().to_string());
    let eval_run = evals::insert_eval_run(
        pool,
        &evals::NewEvalRun {
            id: &eval_run_id,
            suite_id: &suite_id,
            farm_id: &request.farm_id,
            agent_id: &request.agent_id,
            user_id: &request.user_id,
            triggered_via: request.triggered_via,
            client_id: request.client_id.as_ref(),
        },
    )
    .await?;
    let result_id = EvalResultId::new(uuid::Uuid::new_v4().to_string());
    evals::insert_eval_result(pool, &result_id, &eval_run_id, &case.id).await?;
    request.case_prompt = case.prompt.clone();
    request.fixture = request.fixture.or_else(|| case.fixture.clone());
    request.timeout_seconds = request.timeout_seconds.or(case.timeout_secs);
    request.eval_result_id = Some(result_id.clone());
    let farm_id = request.farm_id.clone();
    let user_id = request.user_id.clone();
    let run = match launch(&state, request).await {
        Ok(run) => run,
        Err(err) => {
            fail_eval_run(pool, &eval_run_id, &err).await;
            return Err(err);
        },
    };
    evals::update_eval_result_run(pool, &result_id, &run.id).await?;
    evals::update_eval_run_status(pool, &eval_run_id, EvalRunStatus::Running).await?;
    let settling = Settling {
        farm_id,
        agent,
        user_id,
        case,
        result_id,
        eval_run_id,
    };
    let launched = run.clone();
    tokio::spawn(async move {
        let id = settling.eval_run_id.clone();
        if let Err(err) = settle(&state, judge, &settling, &launched).await {
            fail_eval_run(&state.pool, &id, &err).await;
        }
        set_cancelled(&id, false);
    });
    Ok(CaseLaunched { eval_run, run })
}

struct Settling {
    farm_id: FarmId,
    agent: AgentRow,
    user_id: UserId,
    case: EvalCaseRow,
    result_id: EvalResultId,
    eval_run_id: EvalRunId,
}

async fn settle(
    state: &Arc<FarmState>,
    judge: Option<Arc<dyn EvalJudge>>,
    settling: &Settling,
    launched: &RunRow,
) -> FarmResult<()> {
    let job = CaseJob {
        state,
        judge: judge.as_ref(),
        farm_id: &settling.farm_id,
        agent: &settling.agent,
        user_id: &settling.user_id,
        case: &settling.case,
        result_id: &settling.result_id,
    };
    let pool = &state.pool;
    let outcome =
        record_outcome(pool, &settling.result_id, settle_case(&job, launched).await).await?;
    let mut tally = Tally::default();
    tally.record(&outcome);
    evals::update_eval_run_counts(
        pool,
        &settling.eval_run_id,
        tally.passed,
        tally.failed,
        tally.errored,
    )
    .await?;
    close_eval_run(pool, &settling.eval_run_id, &tally).await
}

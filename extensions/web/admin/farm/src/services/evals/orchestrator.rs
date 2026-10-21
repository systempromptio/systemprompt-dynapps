//! Starting and cancelling an eval run. Starting records the eval run (with
//! the surface that triggered it) and a pending result per case, then hands
//! the cases to a background task that runs them one at a time and closes
//! the eval run with its tallies.
//!
//! Cancellation lives in the eval run's row, not in the process that drives
//! it: cancelling marks the row `cancelled` from whichever replica took the
//! request, the driver reads the row before each case and stops, and closing
//! the run records its tallies without overwriting the `cancelled` status.
//! The row also names the replica driving it, so a booting replica resumes
//! only eval runs whose driver has stopped heartbeating.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use sqlx::PgPool;

use super::case::{CaseJob, CaseOutcome, run_case};
use super::judge::EvalJudge;
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{AgentRow, EvalCaseRow, EvalRunRow};
use crate::repositories::{agents, evals};
use crate::services::runner::report::refresh_report_eval;
use crate::services::runs::cancel;
use crate::state::FarmState;
use crate::types::{
    EvalResultId, EvalResultStatus, EvalRunId, EvalRunStatus, EvalSuiteId, FarmId, FarmRunId,
    TriggeredVia,
};

#[derive(Debug, Default)]
pub(super) struct Tally {
    pub(super) passed: i32,
    pub(super) failed: i32,
    pub(super) errored: i32,
    scores: Vec<i32>,
    cost_microdollars: i64,
}

impl Tally {
    pub(super) fn record(&mut self, outcome: &CaseOutcome) {
        match outcome.status {
            Some(EvalResultStatus::Passed) => self.passed += 1,
            Some(EvalResultStatus::Failed) => self.failed += 1,
            _ => self.errored += 1,
        }
        self.scores.extend(outcome.score);
        self.cost_microdollars += outcome.cost_microdollars;
    }

    pub(super) fn mean_score(&self) -> Option<f64> {
        (!self.scores.is_empty()).then(|| {
            self.scores.iter().map(|s| f64::from(*s)).sum::<f64>() / self.scores.len() as f64
        })
    }
}

/// One request to evaluate a suite: which suite, on whose behalf, from
/// which surface, and optionally only some of its cases.
#[derive(Debug, Clone)]
pub struct EvalLaunch {
    pub farm_id: FarmId,
    pub suite_id: EvalSuiteId,
    pub user_id: UserId,
    pub triggered_via: TriggeredVia,
    pub case_filter: Option<Vec<String>>,
}

struct DriveScope<'a> {
    farm_id: &'a FarmId,
    agent: &'a AgentRow,
    user_id: &'a UserId,
    eval_run_id: &'a EvalRunId,
}

pub async fn start_eval_run(
    state: Arc<FarmState>,
    judge: Option<Arc<dyn EvalJudge>>,
    launch: EvalLaunch,
) -> FarmResult<EvalRunRow> {
    let EvalLaunch {
        farm_id,
        suite_id,
        user_id,
        triggered_via,
        case_filter,
    } = launch;
    let pool = &state.pool;
    let suite = evals::find_eval_suite(pool, &suite_id)
        .await?
        .filter(|s| s.farm_id == farm_id)
        .ok_or_else(|| FarmError::NotFound(format!("eval suite {}", suite_id.as_str())))?;
    let agent = agents::find_agent(pool, &farm_id, &suite.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}", suite.agent_id.as_str())))?;
    let cases: Vec<EvalCaseRow> = evals::list_eval_cases(pool, &suite_id)
        .await?
        .into_iter()
        .filter(|c| case_filter.as_ref().is_none_or(|keep| keep.contains(&c.id)))
        .collect();
    if cases.is_empty() {
        return Err(FarmError::Validation("no eval cases match".to_owned()));
    }
    tracing::info!(suite = suite_id.as_str(), via = %triggered_via, cases = cases.len(), "eval run");
    let eval_run_id = EvalRunId::new(uuid::Uuid::new_v4().to_string());
    let row = evals::insert_eval_run(
        pool,
        &evals::NewEvalRun {
            id: &eval_run_id,
            suite_id: &suite_id,
            farm_id: &farm_id,
            agent_id: &suite.agent_id,
            user_id: &user_id,
            triggered_via,
            client_id: None,
        },
    )
    .await?;
    let mut planned = Vec::with_capacity(cases.len());
    for case in cases {
        let result_id = EvalResultId::new(uuid::Uuid::new_v4().to_string());
        evals::insert_eval_result(pool, &result_id, &eval_run_id, &case.id).await?;
        planned.push((case, result_id));
    }
    tokio::spawn(async move {
        let id = eval_run_id.clone();
        let scope = DriveScope {
            farm_id: &farm_id,
            agent: &agent,
            user_id: &user_id,
            eval_run_id: &id,
        };
        if let Err(err) = drive(&state, judge, &scope, planned).await {
            fail_eval_run(&state.pool, &id, &err).await;
        }
    });
    Ok(row)
}

async fn drive(
    state: &Arc<FarmState>,
    judge: Option<Arc<dyn EvalJudge>>,
    scope: &DriveScope<'_>,
    planned: Vec<(EvalCaseRow, EvalResultId)>,
) -> FarmResult<()> {
    let DriveScope {
        farm_id,
        agent,
        user_id,
        eval_run_id,
    } = *scope;
    let pool = &state.pool;
    evals::claim_eval_run_driver(pool, eval_run_id, &state.instance).await?;
    evals::update_eval_run_status(pool, eval_run_id, EvalRunStatus::Running).await?;
    let mut tally = Tally::default();
    for (case, result_id) in &planned {
        if evals::is_eval_run_cancelled(pool, eval_run_id).await? {
            break;
        }
        let job = CaseJob {
            state,
            judge: judge.as_ref(),
            farm_id,
            agent,
            user_id,
            case,
            result_id,
        };
        let outcome = record_outcome(pool, result_id, run_case(&job).await).await?;
        tally.record(&outcome);
        evals::update_eval_run_counts(pool, eval_run_id, tally.passed, tally.failed, tally.errored)
            .await?;
    }
    close_eval_run(pool, eval_run_id, &tally).await
}

pub(super) async fn record_outcome(
    pool: &PgPool,
    result_id: &EvalResultId,
    settled: FarmResult<CaseOutcome>,
) -> FarmResult<CaseOutcome> {
    match settled {
        Ok(outcome) => Ok(outcome),
        Err(err) => {
            let detail = serde_json::json!([]);
            let message = err.to_string();
            evals::update_eval_result_outcome(
                pool,
                result_id,
                &evals::EvalOutcomeUpdate {
                    status: EvalResultStatus::Errored,
                    assertions: &detail,
                    judge_score: None,
                    judge_rationale: Some(message.as_str()),
                },
            )
            .await?;
            refresh_report_eval(pool, result_id).await;
            Ok(CaseOutcome::default())
        },
    }
}

pub(super) async fn close_eval_run(
    pool: &PgPool,
    eval_run_id: &EvalRunId,
    tally: &Tally,
) -> FarmResult<()> {
    evals::update_eval_run_finished(
        pool,
        eval_run_id,
        EvalRunStatus::Completed,
        tally.mean_score(),
        tally.cost_microdollars,
    )
    .await?;
    Ok(())
}

pub(super) async fn fail_eval_run(pool: &PgPool, id: &EvalRunId, err: &FarmError) {
    tracing::error!(eval_run = id.as_str(), error = %err, "eval run failed");
    if let Err(e) = evals::update_eval_run_status(pool, id, EvalRunStatus::Failed).await {
        tracing::error!(eval_run = id.as_str(), error = %e, "eval run status not recorded");
    }
}

pub async fn cancel_eval_run(state: &FarmState, eval_run_id: &EvalRunId) -> FarmResult<()> {
    evals::update_eval_run_status(&state.pool, eval_run_id, EvalRunStatus::Cancelled).await?;
    let Some(eval_run) = evals::find_eval_run(&state.pool, eval_run_id).await? else {
        return Ok(());
    };
    let results = evals::list_eval_results(&state.pool, eval_run_id).await?;
    let running = results
        .iter()
        .filter(|r| r.status == EvalResultStatus::Running.as_str())
        .filter_map(|r| r.run_id.as_deref());
    for run_id in running {
        cancel(state, &FarmRunId::new(run_id), &eval_run.user_id).await?;
    }
    Ok(())
}

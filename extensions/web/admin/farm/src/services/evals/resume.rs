//! Boot-time recovery of eval runs whose driver died with the API.
//!
//! The orchestrator and case triggers judge from a spawned task; a
//! restart drops that task and leaves the eval run `running` forever.
//!
//! At boot every eval run still `queued` or `running` is planned case by
//! case with [`plan_resume`]: a case with a farm run is settled (judged now
//! if the run is terminal, else watched until it is), a case never launched
//! is launched, and an eval run older than [`MAX_EVAL_AGE`] is abandoned —
//! its open cases errored with the reason and the eval run marked failed.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use super::case::{CaseJob, CaseOutcome, run_case, settle_case};
use super::orchestrator::{Tally, close_eval_run, fail_eval_run, record_outcome};
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{EvalResultRow, EvalRunRow};
use crate::repositories::{agents, evals, runs};
use crate::state::FarmState;
use crate::types::{EvalResultStatus, EvalRunStatus, FarmRunId};

pub const MAX_EVAL_AGE: Duration = Duration::hours(6);
const SCAN_LIMIT: i64 = 1_000;

/// What boot does with one open case of an interrupted eval run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeAction {
    Settle { result_id: String, run_id: String },
    Launch { result_id: String },
    Abandon { result_id: String, reason: String },
}

#[must_use]
pub fn select_interrupted(rows: &[EvalRunRow]) -> Vec<&EvalRunRow> {
    rows.iter()
        .filter(|r| {
            matches!(
                EvalRunStatus::parse(&r.status),
                Some(EvalRunStatus::Queued | EvalRunStatus::Running)
            )
        })
        .collect()
}

#[must_use]
pub fn plan_resume(
    eval_run: &EvalRunRow,
    results: &[EvalResultRow],
    now: DateTime<Utc>,
) -> Vec<ResumeAction> {
    let since = eval_run.started_at.unwrap_or(eval_run.created_at);
    let expired = now - since > MAX_EVAL_AGE;
    results
        .iter()
        .filter(|r| {
            matches!(
                EvalResultStatus::parse(&r.status),
                Some(EvalResultStatus::Pending | EvalResultStatus::Running)
            )
        })
        .map(|r| {
            let result_id = r.id.as_str().to_owned();
            if expired {
                ResumeAction::Abandon {
                    result_id,
                    reason: format!(
                        "eval abandoned: the API restarted and the eval run was open over {}h",
                        MAX_EVAL_AGE.num_hours()
                    ),
                }
            } else if let Some(run_id) = &r.run_id {
                ResumeAction::Settle {
                    result_id,
                    run_id: run_id.clone(),
                }
            } else {
                ResumeAction::Launch { result_id }
            }
        })
        .collect()
}

pub async fn resume_eval_runs(state: &Arc<FarmState>) -> usize {
    let rows = match evals::list_recent_eval_runs(&state.pool, None, None, SCAN_LIMIT).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = %e, "interrupted eval runs not listed");
            return 0;
        },
    };
    let open: Vec<EvalRunRow> = select_interrupted(&rows).into_iter().cloned().collect();
    let count = open.len();
    for eval_run in open {
        let state = Arc::clone(state);
        tokio::spawn(async move {
            if let Err(err) = resume_one(&state, &eval_run).await {
                fail_eval_run(&state.pool, &eval_run.id, &err).await;
            }
        });
    }
    count
}

async fn resume_one(state: &Arc<FarmState>, eval_run: &EvalRunRow) -> FarmResult<()> {
    let pool = &state.pool;
    let results = evals::list_eval_results(pool, &eval_run.id).await?;
    let plan = plan_resume(eval_run, &results, Utc::now());
    tracing::info!(
        eval_run = eval_run.id.as_str(),
        cases = plan.len(),
        "resuming eval run"
    );
    let agent = agents::find_agent(pool, &eval_run.farm_id, &eval_run.agent_id)
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("agent {}", eval_run.agent_id.as_str())))?;
    let cases = evals::list_eval_cases(pool, &eval_run.suite_id).await?;
    let judge = state.judge.judge_for(None, &eval_run.user_id);
    let mut abandoned = false;
    for action in plan {
        let (result_id, settled) = match action {
            ResumeAction::Abandon { result_id, reason } => {
                abandoned = true;
                let row = find_result(&results, &result_id)?;
                let err = FarmError::Spawn(reason);
                record_outcome(pool, &row.id, Err(err)).await?;
                continue;
            },
            ResumeAction::Settle { result_id, run_id } => (result_id, Some(run_id)),
            ResumeAction::Launch { result_id } => (result_id, None),
        };
        let row = find_result(&results, &result_id)?;
        let case = cases
            .iter()
            .find(|c| c.id == row.case_id)
            .ok_or_else(|| FarmError::NotFound(format!("eval case {}", row.case_id)))?;
        let job = CaseJob {
            state,
            judge: judge.as_ref(),
            farm_id: &eval_run.farm_id,
            agent: &agent,
            user_id: &eval_run.user_id,
            case,
            result_id: &row.id,
        };
        let outcome = match settled {
            Some(run_id) => settle_existing(&job, &run_id).await,
            None => run_case(&job).await,
        };
        record_outcome(pool, &row.id, outcome).await?;
    }
    let tally = tally_of(&evals::list_eval_results(pool, &eval_run.id).await?);
    evals::update_eval_run_counts(
        pool,
        &eval_run.id,
        tally.passed,
        tally.failed,
        tally.errored,
    )
    .await?;
    if abandoned {
        evals::update_eval_run_status(pool, &eval_run.id, EvalRunStatus::Failed).await?;
        return Ok(());
    }
    close_eval_run(pool, &eval_run.id, &tally).await
}

async fn settle_existing(job: &CaseJob<'_>, run_id: &str) -> FarmResult<CaseOutcome> {
    let (run, _) = runs::find_run_with_agent_run(&job.state.pool, &FarmRunId::new(run_id))
        .await?
        .ok_or_else(|| FarmError::NotFound(format!("run {run_id}")))?;
    settle_case(job, &run).await
}

fn find_result<'a>(results: &'a [EvalResultRow], id: &str) -> FarmResult<&'a EvalResultRow> {
    results
        .iter()
        .find(|r| r.id.as_str() == id)
        .ok_or_else(|| FarmError::NotFound(format!("eval result {id}")))
}

fn tally_of(results: &[EvalResultRow]) -> Tally {
    let mut tally = Tally::default();
    for row in results {
        tally.record(&CaseOutcome {
            status: EvalResultStatus::parse(&row.status),
            score: row.judge_score.map(|s| s.round() as i32),
            cost_microdollars: 0,
        });
    }
    tally
}

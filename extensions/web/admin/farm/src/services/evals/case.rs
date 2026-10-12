//! One eval case end to end: launch its farm run, wait for a terminal
//! status, check its assertions, ask the judge when the case has a rubric,
//! and record the outcome on the case's result row.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use systemprompt::identifiers::UserId;

use super::assertions::{AssertionContext, AssertionResult, evaluate_assertions};
use super::judge::{EvalJudge, JudgeInput};
use super::transcript::{final_result_text, render_transcript, truncate_utf8};
use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{AgentRow, EvalCaseRow, RunEventRow, RunRow};
use crate::repositories::{evals, run_events, runs};
use crate::services::farm_sync::eval_yaml::{Assertion, DEFAULT_JUDGE_THRESHOLD};
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{EvalResultId, EvalResultStatus, FarmId, FarmRunId, RunStatus, TriggeredVia};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const WAIT_GRACE_SECONDS: u64 = 60;
const OUTPUT_CAP_BYTES: usize = 16 * 1024;

pub(crate) struct CaseJob<'a> {
    pub state: &'a Arc<FarmState>,
    pub judge: Option<&'a Arc<dyn EvalJudge>>,
    pub farm_id: &'a FarmId,
    pub agent: &'a AgentRow,
    pub user_id: &'a UserId,
    pub case: &'a EvalCaseRow,
    pub result_id: &'a EvalResultId,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CaseOutcome {
    pub status: Option<EvalResultStatus>,
    pub score: Option<i32>,
    pub cost_microdollars: i64,
}

pub(crate) async fn run_case(job: &CaseJob<'_>) -> FarmResult<CaseOutcome> {
    let pool = &job.state.pool;
    let timeout = job
        .case
        .timeout_secs
        .and_then(|t| u64::try_from(t).ok())
        .unwrap_or(job.state.limits.default_timeout_seconds);
    let launched = launch(
        job.state,
        LaunchRequest {
            farm_id: job.farm_id.clone(),
            agent_id: job.agent.id.clone(),
            user_id: job.user_id.clone(),
            triggered_via: TriggeredVia::Eval,
            prompt: None,
            case_prompt: job.case.prompt.clone(),
            fixture: job.case.fixture.clone(),
            model: None,
            max_turns: None,
            max_budget_usd: None,
            timeout_seconds: i32::try_from(timeout).ok(),
            eval_result_id: Some(job.result_id.clone()),
        },
    )
    .await?;
    evals::update_eval_result_run(pool, job.result_id, &launched.id).await?;
    let run = wait_for_terminal(job.state, &launched.id, timeout + WAIT_GRACE_SECONDS).await?;
    let events = run_events::list_run_events(pool, &run.id).await?;
    let workspace = workspace_of(&run);
    let final_text = final_result_text(&events);
    let assertions: Vec<Assertion> = serde_json::from_value(job.case.assertions.clone())
        .map_err(|e| FarmError::Validation(format!("case {} assertions: {e}", job.case.id)))?;
    let results = evaluate_assertions(
        &assertions,
        &AssertionContext {
            run: &run,
            events: &events,
            workspace: &workspace,
            agent_tools: &job.agent.tools,
            output_path: job.agent.output_path.as_deref(),
            final_text: final_text.as_deref(),
        },
    );
    let verdict = judge_case(job, &run, &events, &workspace).await;
    let (status, score, rationale) = settle(&run, &results, verdict);
    let assertions_json = serde_json::to_value(&results)
        .map_err(|e| FarmError::Validation(format!("assertion results: {e}")))?;
    evals::update_eval_result_outcome(
        pool,
        job.result_id,
        &evals::EvalOutcomeUpdate {
            status,
            assertions: &assertions_json,
            judge_score: score.map(f64::from),
            judge_rationale: rationale.as_deref(),
        },
    )
    .await?;
    Ok(CaseOutcome {
        status: Some(status),
        score,
        cost_microdollars: run.cost_microdollars.unwrap_or(0),
    })
}

type Verdict = Option<Result<(i32, bool, String), String>>;

fn settle(
    run: &RunRow,
    results: &[AssertionResult],
    verdict: Verdict,
) -> (EvalResultStatus, Option<i32>, Option<String>) {
    let succeeded = RunStatus::parse(&run.status) == Some(RunStatus::Succeeded);
    let asserted = results.iter().all(|r| r.passed);
    match verdict {
        Some(Err(err)) => (
            EvalResultStatus::Errored,
            None,
            Some(format!("judge failed: {err}")),
        ),
        _ if !succeeded => {
            let why = run
                .error
                .clone()
                .unwrap_or_else(|| format!("run {}", run.status));
            let score = verdict.and_then(Result::ok).map(|(s, _, _)| s);
            (EvalResultStatus::Errored, score, Some(why))
        },
        Some(Ok((score, judged, rationale))) => {
            let status = if asserted && judged {
                EvalResultStatus::Passed
            } else {
                EvalResultStatus::Failed
            };
            (status, Some(score), Some(rationale))
        },
        None if asserted => (EvalResultStatus::Passed, None, None),
        None => (EvalResultStatus::Failed, None, None),
    }
}

async fn judge_case(
    job: &CaseJob<'_>,
    run: &RunRow,
    events: &[RunEventRow],
    workspace: &Path,
) -> Verdict {
    let rubric = job.case.judge_rubric.as_deref()?;
    let judge = job.judge?;
    let mut transcript = render_transcript(&run.prompt, events);
    if let Some(rel) = job.agent.output_path.as_deref()
        && let Ok(body) = std::fs::read_to_string(workspace.join(rel))
    {
        let body = truncate_utf8(body, OUTPUT_CAP_BYTES);
        transcript = format!("OUTPUT FILE {rel}:\n{body}\n\n{transcript}");
    }
    let threshold = job.case.judge_threshold.map_or_else(
        || i32::from(DEFAULT_JUDGE_THRESHOLD),
        |t| t.round().clamp(0.0, 100.0) as i32,
    );
    let input = JudgeInput {
        rubric: rubric.to_owned(),
        case_prompt: run.prompt.clone(),
        transcript,
        diff_summary: run
            .diff_summary
            .as_ref()
            .map_or_else(|| "(none recorded)".to_owned(), ToString::to_string),
        threshold,
    };
    Some(
        judge
            .judge(&input)
            .await
            .map(|v| (v.score, v.passed, v.rationale))
            .map_err(|e| e.to_string()),
    )
}

fn workspace_of(run: &RunRow) -> PathBuf {
    let root = PathBuf::from(run.run_dir.clone().unwrap_or_default());
    let nested = root.join("workspace");
    if nested.is_dir() { nested } else { root }
}

async fn wait_for_terminal(
    state: &Arc<FarmState>,
    run_id: &FarmRunId,
    budget_seconds: u64,
) -> FarmResult<RunRow> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(budget_seconds);
    loop {
        let run = runs::find_run(&state.pool, run_id)
            .await?
            .ok_or_else(|| FarmError::NotFound(format!("run {}", run_id.as_str())))?;
        if RunStatus::parse(&run.status).is_some_and(RunStatus::is_terminal) {
            return Ok(run);
        }
        if tokio::time::Instant::now() >= deadline {
            if let Some(runner) = state.runner() {
                runner.request_cancel(run_id);
            }
            return Err(FarmError::Spawn(format!(
                "run {} did not finish within {budget_seconds}s",
                run_id.as_str()
            )));
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

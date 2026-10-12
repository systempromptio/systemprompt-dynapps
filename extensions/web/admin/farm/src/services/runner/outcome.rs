//! Maps how a child ended, its final `result` event and the workspace diff
//! onto the [`RunOutcome`] the run row records.

use super::diff::WorkspaceDiff;
use super::stream_json::ResultSummary;
use super::supervise::Ended;
use crate::repositories::filters::RunOutcome;
use crate::types::RunStatus;

#[must_use]
pub fn failed(error: impl Into<String>) -> RunOutcome {
    terminal(RunStatus::Failed, Some(error.into()))
}

#[must_use]
pub const fn terminal(status: RunStatus, error: Option<String>) -> RunOutcome {
    RunOutcome {
        status,
        context_id: None,
        duration_ms: None,
        num_turns: None,
        input_tokens: None,
        output_tokens: None,
        cost_microdollars: None,
        result_subtype: None,
        exit_code: None,
        error,
        diff_summary: None,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OutcomeInputs<'a> {
    pub ended: &'a Ended,
    pub result: Option<&'a ResultSummary>,
    pub stderr_tail: &'a str,
    pub diff: &'a WorkspaceDiff,
    pub wall_ms: i64,
}

#[must_use]
pub fn run_outcome(inputs: OutcomeInputs<'_>) -> RunOutcome {
    let (status, exit_code, error) = classify(inputs);
    let result = inputs.result;
    let usage = result.map(|r| r.usage).unwrap_or_default();
    RunOutcome {
        status,
        context_id: None,
        duration_ms: Some(result.and_then(|r| r.duration_ms).unwrap_or(inputs.wall_ms)),
        num_turns: result.and_then(|r| r.num_turns).map(|n| n as i32),
        input_tokens: result
            .map(|_| usage.input + usage.cache_read_input + usage.cache_creation_input),
        output_tokens: result.map(|_| usage.output),
        cost_microdollars: result
            .and_then(|r| r.total_cost_usd)
            .map(|usd| (usd * 1_000_000.0).round() as i64),
        result_subtype: result.and_then(|r| r.subtype.clone()),
        exit_code,
        error,
        diff_summary: serde_json::to_value(inputs.diff).ok(),
    }
}

fn classify(inputs: OutcomeInputs<'_>) -> (RunStatus, Option<i32>, Option<String>) {
    let tail = inputs.stderr_tail.trim();
    let stderr = (!tail.is_empty()).then(|| tail.to_owned());
    match inputs.ended {
        Ended::Cancelled => (RunStatus::Cancelled, None, Some("cancelled".to_owned())),
        Ended::TimedOut => (RunStatus::TimedOut, None, Some("timed out".to_owned())),
        Ended::WaitFailed(err) => (RunStatus::Failed, None, Some(err.clone())),
        Ended::Exited(code) => match inputs.result {
            Some(result) if result.succeeded() => (RunStatus::Succeeded, *code, None),
            Some(result) => {
                let reason = result
                    .result
                    .clone()
                    .or(stderr)
                    .unwrap_or_else(|| result.subtype.clone().unwrap_or_default());
                (RunStatus::Failed, *code, Some(reason))
            },
            None => {
                let reason = stderr
                    .unwrap_or_else(|| format!("claude exited ({code:?}) without a result event"));
                (RunStatus::Failed, *code, Some(reason))
            },
        },
    }
}

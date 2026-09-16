//! Serializable views behind the experiment pages: one experiment, one
//! execution, one approval wait, and the index totals. Every badge tone is
//! decided here so the templates carry no status logic.

use serde::Serialize;
use systemprompt::evaluation::experiments::records::{
    ExecutionStatus, ExperimentRecord, ExperimentStatus,
};
use systemprompt::identifiers::{EvalExecutionId, EvalExperimentId, EvalRevisionId};

#[derive(Serialize)]
pub(super) struct ExperimentView {
    pub(super) id: EvalExperimentId,
    pub(super) name: String,
    pub(super) status: ExperimentStatus,
    pub(super) cases: usize,
    pub(super) variants: usize,
    pub(super) repetitions: u32,
    pub(super) cap: String,
    pub(super) settled: String,
    pub(super) reserved: String,
    pub(super) frozen: bool,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) executions: usize,
}

// Why: the badge tone is a presentation decision; deciding it here keeps the
// match over every status variant exhaustive at compile time, so a variant
// without a tone is a build error rather than an untoned badge.
const fn status_presentation(status: ExperimentStatus) -> (&'static str, &'static str) {
    match status {
        ExperimentStatus::Queued => ("Queued", "muted"),
        ExperimentStatus::Running => ("Running", "info"),
        ExperimentStatus::Completed => ("Completed", "ok"),
        ExperimentStatus::Cancelled => ("Cancelled", "warn"),
        ExperimentStatus::Blocked => ("Blocked", "err"),
    }
}

impl From<ExperimentRecord> for ExperimentView {
    fn from(row: ExperimentRecord) -> Self {
        let (status_label, status_tone) = status_presentation(row.status);
        Self {
            status_label,
            status_tone,
            executions: row.spec.cases.len()
                * row.spec.variants.len()
                * usize::try_from(row.spec.repetitions).unwrap_or(1).max(1),
            id: row.id,
            name: row.spec.name,
            status: row.status,
            cases: row.spec.cases.len(),
            variants: row.spec.variants.len(),
            repetitions: row.spec.repetitions,
            cap: super::dollars(row.accounting.cap),
            settled: super::dollars(row.accounting.settled),
            reserved: super::dollars(row.accounting.reserved),
            frozen: row.accounting.frozen,
        }
    }
}

#[derive(Serialize)]
pub(super) struct ExecutionView {
    pub(super) id: EvalExecutionId,
    pub(super) case_revision: EvalRevisionId,
    pub(super) variant: i32,
    pub(super) repetition: i32,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) summary: Option<String>,
}

// Why: the detail page needs the same compile-checked exhaustive match the
// index gets, so the execution mapping is the one the handler calls.
pub(super) const fn execution_presentation(
    status: ExecutionStatus,
) -> (&'static str, &'static str) {
    match status {
        ExecutionStatus::Queued => ("Queued", "muted"),
        ExecutionStatus::Running => ("Running", "info"),
        ExecutionStatus::AwaitingApproval => ("Awaiting approval", "warn"),
        ExecutionStatus::Completed => ("Completed", "ok"),
        ExecutionStatus::Error => ("Error", "err"),
        ExecutionStatus::Cancelled => ("Cancelled", "muted"),
        ExecutionStatus::Blocked => ("Blocked", "err"),
        ExecutionStatus::BudgetExhausted => ("Budget exhausted", "err"),
    }
}

#[derive(Default, Serialize)]
pub(super) struct ExperimentTotals {
    pub(super) recorded: usize,
    pub(super) running: usize,
    pub(super) completed: usize,
    pub(super) executions: usize,
    pub(super) settled: String,
    pub(super) reserved: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub(super) struct ApprovalView {
    pub(super) id: String,
    pub(super) execution_id: EvalExecutionId,
    pub(super) status: String,
    pub(super) precondition_digest: String,
    // JSON: retained approval operations are deliberately arbitrary tool payloads.
    pub(super) operation: serde_json::Value,
    pub(super) expires_at: chrono::DateTime<chrono::Utc>,
}

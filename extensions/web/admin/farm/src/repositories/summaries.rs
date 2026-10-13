//! Aggregates the farm repositories compute rather than read row for row:
//! run statistics, the daily sparkline, a run's gateway requests and the
//! eval matrix.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::types::{EvalRunId, EvalSuiteId, FarmAgentId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct RunStats {
    pub total: i64,
    pub running: i64,
    pub queued: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub timed_out: i64,
    pub cancelled: i64,
    pub cost_microdollars: i64,
    pub p50_duration_ms: Option<f64>,
    pub p95_duration_ms: Option<f64>,
    pub tool_calls: i64,
    pub gateway_requests: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DailyRunCount {
    pub day: NaiveDate,
    pub runs: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunRequestRow {
    pub request_id: String,
    pub model: Option<String>,
    pub latency_ms: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatrixCellRow {
    pub suite_id: EvalSuiteId,
    pub case_id: String,
    pub agent_id: FarmAgentId,
    pub eval_run_id: EvalRunId,
    pub status: String,
    pub passed: bool,
    pub judge_score: Option<f64>,
    pub rationale: Option<String>,
    pub finished_at: Option<DateTime<Utc>>,
}

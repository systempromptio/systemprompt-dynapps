//! Eval rows: the suite synced per agent, its cases, one eval run per
//! launch, and one result per case linked to the farm run that produced it.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::types::{EvalResultId, EvalRunId, EvalSuiteId, FarmAgentId, FarmId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct EvalSuiteRow {
    pub id: EvalSuiteId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub source_path: String,
    pub case_count: i32,
    pub synced_commit_sha: String,
    pub updated_at: DateTime<Utc>,
}

// JSON: fixed-shape document — `assertions` is the case's closed assertion
// list from the eval file, stored as JSONB and decoded by the evals service.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct EvalCaseRow {
    pub suite_id: EvalSuiteId,
    pub id: String,
    pub prompt: Option<String>,
    pub fixture: Option<String>,
    pub timeout_secs: Option<i32>,
    pub assertions: serde_json::Value,
    pub judge_rubric: Option<String>,
    pub judge_threshold: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct EvalRunRow {
    pub id: EvalRunId,
    pub suite_id: EvalSuiteId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub user_id: UserId,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub passed: i32,
    pub failed: i32,
    pub errored: i32,
    pub mean_score: Option<f64>,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
    pub triggered_via: String,
}

/// One agent's eval runs on one day: how many ran, how many of their cases
/// passed out of how many were decided, the mean judge score and the spend.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct EvalDayRow {
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub day: NaiveDate,
    pub runs: i64,
    pub passed: i64,
    pub decided: i64,
    pub mean_score: Option<f64>,
    pub cost_microdollars: i64,
}

// JSON: fixed-shape document — `assertions` holds each assertion's outcome
// as the evals service wrote it, stored as JSONB and rendered per case.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct EvalResultRow {
    pub id: EvalResultId,
    pub eval_run_id: EvalRunId,
    pub case_id: String,
    pub run_id: Option<String>,
    pub status: String,
    pub assertions: serde_json::Value,
    pub judge_score: Option<f64>,
    pub judge_rationale: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

//! Row shapes for the per-client report: client rollups, daily outcomes,
//! per-agent outcomes, recent runs with their completion report, and the
//! run detail's report, identity and governance rows.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::types::{ClientId, FarmAgentId, FarmId, FarmRunId};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClientSummaryRow {
    pub client_id: ClientId,
    pub name: String,
    pub slug: String,
    pub odoo_partner_id: Option<i64>,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub active: i64,
    pub people: i64,
    pub cost_microdollars: i64,
    pub completed: i64,
    pub reported: i64,
    pub eval_mean: Option<f64>,
    pub eval_scored: i64,
    pub infra_failures: i64,
    pub last_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct DailyOutcomeRow {
    pub day: NaiveDate,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub cost_microdollars: i64,
    pub eval_mean: Option<f64>,
    pub eval_scored: i64,
    pub infra_failures: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentOutcomeRow {
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub cost_microdollars: i64,
    pub completed: i64,
    pub eval_mean: Option<f64>,
    pub eval_scored: i64,
    pub infra_failures: i64,
    pub last_run_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReportRunRow {
    pub run_id: FarmRunId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub client_id: Option<ClientId>,
    pub client_name: Option<String>,
    pub user_id: UserId,
    pub status: String,
    pub triggered_via: String,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub completed: Option<bool>,
    pub summary: Option<String>,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunReportViewRow {
    pub run_id: FarmRunId,
    pub completed: bool,
    pub summary: Option<String>,
    pub final_message: Option<String>,
    // JSON: JSONB columns the runner observer writes; their item shapes
    // ({path, bytes, sha256} and {tool, model, record_ids, decision, at})
    // are read field by field in the Report tab.
    pub outputs: serde_json::Value,
    pub odoo_writes: serde_json::Value,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunIdentityRow {
    pub client_id: Option<ClientId>,
    pub client_name: Option<String>,
    pub launcher_name: String,
    pub run_as_id: Option<UserId>,
    pub run_as_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunDecisionRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub actor_kind: String,
    pub actor_id: String,
    pub user_id: UserId,
}

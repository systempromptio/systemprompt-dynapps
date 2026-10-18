//! Output contracts for the agent-farm tools. Each implements core's
//! `McpOutputSchema` so the executor can wrap it as an artifact; the
//! artifact type names the tool family rather than a core table shape.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt::mcp::McpOutputSchema;
use systemprompt_web_admin_core::repositories::agent_run_facts::{RunFacts, RunStats};

macro_rules! farm_output {
    ($ty:ty, $name:literal) => {
        impl McpOutputSchema for $ty {
            fn artifact_type() -> &'static str {
                $name
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunStartOutput {
    pub run_id: String,
    pub origin: String,
    pub farm_run_id: String,
    pub status: String,
    pub eval_run_id: Option<String>,
    pub hint: String,
}
farm_output!(RunStartOutput, "agent_run_start");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunStatusOutput {
    pub run_id: String,
    pub origin_kind: String,
    pub origin_id: String,
    pub user_id: UserId,
    pub host: String,
    pub status: String,
    pub terminal: bool,
    pub error_class: Option<String>,
    pub error: Option<String>,
    pub attempt: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub exit_code: Option<i32>,
    pub cancel_requested: bool,
    pub client: Option<String>,
    pub report: Option<super::outputs_report::RunReportOutput>,
}
farm_output!(RunStatusOutput, "agent_run_status");

// JSON: a run event's payload is the host's own stream shape (tool inputs
// and structured answers), relayed as core stored it.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunEventOutput {
    pub seq: i64,
    pub kind: String,
    pub tool_name: Option<String>,
    pub tool_use_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunEventsOutput {
    pub run_id: String,
    pub events: Vec<RunEventOutput>,
    pub returned: usize,
    pub total_events: i64,
    pub next_after_seq: i64,
    pub exhausted: bool,
}
farm_output!(RunEventsOutput, "agent_run_events");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunCancelOutput {
    pub run_id: String,
    pub farm_run_id: String,
    pub cancelled: bool,
    pub message: String,
}
farm_output!(RunCancelOutput, "agent_run_cancel");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunListOutput {
    pub scope: String,
    pub runs: Vec<RunFacts>,
    pub returned: usize,
    pub hint: String,
}
farm_output!(RunListOutput, "agent_run_list");

/// One assertion's recorded outcome. No judge field exists here on purpose.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AssertionOutcome {
    pub kind: String,
    pub passed: bool,
    pub detail: String,
}

/// A farm's own record of one run, beside core's row.
#[cfg(feature = "farm")]
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FarmRunExtras {
    pub farm_run_id: String,
    pub farm: String,
    pub agent: String,
    pub triggered_via: String,
    pub prompt: String,
    pub model: Option<String>,
    pub eval_result_id: Option<String>,
    pub eval_run_id: Option<String>,
    pub eval_case_id: Option<String>,
    pub eval_status: Option<String>,
    pub assertions: Vec<AssertionOutcome>,
}

/// What the origin knows about a run beyond core's row.
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "origin", rename_all = "snake_case")]
pub enum RunExtras {
    None,
    #[cfg(feature = "farm")]
    Farm(Box<FarmRunExtras>),
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunFactsOutput {
    pub facts: RunFacts,
    pub extras: RunExtras,
}
farm_output!(RunFactsOutput, "agent_run_facts");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunStatsOutput {
    pub scope: String,
    pub window: String,
    pub farm: String,
    pub stats: RunStats,
}
farm_output!(RunStatsOutput, "agent_run_stats");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EvalRunOutput {
    pub eval_run_id: String,
    pub farm: String,
    pub suite: String,
    pub agent: String,
    pub status: String,
    pub hint: String,
}
farm_output!(EvalRunOutput, "agent_farm_eval_run");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EvalCellOutput {
    pub farm: String,
    pub suite: String,
    pub case_id: String,
    pub agent: String,
    pub eval_run_id: String,
    pub status: String,
    pub passed: bool,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EvalDayOutput {
    pub farm: String,
    pub agent: String,
    pub day: String,
    pub runs: i64,
    pub passed: i64,
    pub decided: i64,
    pub pass_rate: Option<f64>,
    pub mean_score: Option<f64>,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct EvalResultsOutput {
    pub rows: Vec<EvalCellOutput>,
    pub returned: usize,
    pub passed: usize,
    pub failed: usize,
    pub errored: usize,
    pub history: Vec<EvalDayOutput>,
}
farm_output!(EvalResultsOutput, "agent_farm_eval_results");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CatalogItemOutput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CatalogSectionOutput {
    pub entity_type: String,
    pub label: String,
    pub items: Vec<CatalogItemOutput>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CatalogOutput {
    pub user_id: UserId,
    pub roles: Vec<String>,
    pub project_ids: Vec<String>,
    pub sections: Vec<CatalogSectionOutput>,
}
farm_output!(CatalogOutput, "agent_farm_catalog");

//! Inputs to the farm repositories: the rows a sync or a launch writes, the
//! farm facts a finished run records, and the filter and sort the run list
//! reads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{AgentRunId, ContextId, MarketplaceId, PluginId, UserId};
use systemprompt_web_shared::ProjectId;

use crate::config::RunAsMode;
use crate::types::{
    ClientId, EvalResultId, EvalSuiteId, FarmAgentId, FarmId, FarmRunId, RunStatus, TriggeredVia,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFarm {
    pub id: FarmId,
    pub name: String,
    pub description: Option<String>,
    pub marketplace_id: MarketplaceId,
    pub source_repo: String,
    pub source_ref: String,
    pub source_subdir: String,
    pub permission_mode: String,
    pub timeout_seconds: i32,
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub model: Option<String>,
    pub run_as_user_id: Option<UserId>,
    pub run_as_mode: Option<RunAsMode>,
    pub project_id: Option<ProjectId>,
    pub client_id: Option<ClientId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAgent {
    pub farm_id: FarmId,
    pub id: FarmAgentId,
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub host: String,
    pub tools: Vec<String>,
    pub plugin_id: PluginId,
    pub skills: Vec<String>,
    pub entry_prompt: String,
    pub fixture: Option<String>,
    pub max_turns: Option<i32>,
    pub max_budget_usd: Option<f64>,
    pub output_path: Option<String>,
    pub output_format: Option<String>,
    pub source_path: String,
    pub body: String,
    pub synced_commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFixture {
    pub farm_id: FarmId,
    pub name: String,
    pub file_count: i32,
    pub bytes: i64,
    pub stored_path: String,
    pub synced_commit_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRun {
    pub id: FarmRunId,
    pub agent_run_id: AgentRunId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub user_id: UserId,
    pub triggered_via: TriggeredVia,
    pub host: String,
    pub attempt: i32,
    pub prompt: String,
    pub fixture: Option<String>,
    pub model: Option<String>,
    pub permission_mode: String,
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub timeout_seconds: i32,
    pub source_commit_sha: Option<String>,
    pub eval_result_id: Option<EvalResultId>,
    pub client_id: Option<ClientId>,
}

/// The farm facts a finished run records; the executor's own outcome is on
/// the `agent_runs` row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunFinish {
    pub status: RunStatus,
    pub attempt: i32,
    pub error_class: Option<String>,
    pub cost_microdollars: i64,
    pub event_count: i32,
    pub tool_call_count: i32,
    pub gateway_request_count: i32,
    pub context_id: Option<ContextId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvalSuite {
    pub id: EvalSuiteId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub source_path: String,
    pub case_count: i32,
    pub synced_commit_sha: String,
}

// JSON: fixed-shape document — the case's closed assertion list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvalCase {
    pub id: String,
    pub prompt: Option<String>,
    pub fixture: Option<String>,
    pub timeout_secs: Option<i32>,
    pub assertions: serde_json::Value,
    pub judge_rubric: Option<String>,
    pub judge_threshold: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunFilter {
    pub farm_id: Option<FarmId>,
    pub agent_id: Option<FarmAgentId>,
    pub user_id: Option<UserId>,
    pub status: Option<RunStatus>,
    pub triggered_via: Option<TriggeredVia>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunSort {
    #[default]
    CreatedDesc,
    CreatedAsc,
    DurationDesc,
    CostDesc,
    ToolCallsDesc,
}

impl RunSort {
    #[must_use]
    pub const fn sql_key(self) -> &'static str {
        match self {
            Self::CreatedDesc => "created_desc",
            Self::CreatedAsc => "created_asc",
            Self::DurationDesc => "duration_desc",
            Self::CostDesc => "cost_desc",
            Self::ToolCallsDesc => "tool_calls_desc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageWindow {
    pub limit: i64,
    pub offset: i64,
}

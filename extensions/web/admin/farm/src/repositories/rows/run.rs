//! The `agent_farm_runs` row: the farm's facts about one run. The executor's
//! live state (lease, pid, run dir, tokens, exit code, diff) lives on the
//! `agent_runs` row it references by `agent_run_id`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentRunId, ContextId, UserId};

use crate::types::{FarmAgentId, FarmId, FarmRunId};

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct RunRow {
    pub id: FarmRunId,
    pub agent_run_id: AgentRunId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub user_id: UserId,
    pub triggered_via: String,
    pub host: String,
    pub status: String,
    pub attempt: i32,
    pub prompt: String,
    pub fixture: Option<String>,
    pub model: Option<String>,
    pub permission_mode: String,
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub timeout_seconds: i32,
    pub source_commit_sha: Option<String>,
    pub context_id: Option<ContextId>,
    pub error_class: Option<String>,
    pub cost_microdollars: i64,
    pub event_count: i32,
    pub tool_call_count: i32,
    pub gateway_request_count: i32,
    pub eval_result_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub cancel_requested_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

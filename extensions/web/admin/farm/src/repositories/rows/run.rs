//! The `agent_farm_runs` row: one headless Claude Code process, from queue
//! to terminal status, with the gateway identifiers that join it to the
//! request ledger and the conversation viewer.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};

use crate::types::{FarmAgentId, FarmId, FarmRunId};

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct RunRow {
    pub id: FarmRunId,
    pub farm_id: FarmId,
    pub agent_id: FarmAgentId,
    pub user_id: UserId,
    pub triggered_via: String,
    pub status: String,
    pub prompt: String,
    pub fixture: Option<String>,
    pub model: Option<String>,
    pub permission_mode: String,
    pub max_turns: i32,
    pub max_budget_usd: f64,
    pub timeout_seconds: i32,
    pub source_commit_sha: Option<String>,
    pub run_dir: Option<String>,
    pub gateway_session_id: Option<String>,
    pub claude_session_id: Option<String>,
    pub context_id: Option<ContextId>,
    pub api_key_id: Option<String>,
    pub pid: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub num_turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub result_subtype: Option<String>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    // JSON: fixed-shape document — the workspace diff the runner computes
    // after exit, stored as JSONB and rendered as-is.
    pub diff_summary: Option<serde_json::Value>,
    pub eval_result_id: Option<String>,
    pub cancel_requested_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

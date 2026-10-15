//! Row shapes for the `agent_farm_*` tables and the
//! `agent_farm_runs_by_user` view, column for column as
//! `extensions/web/schema/52_agent_farm.sql` declares them.

mod eval;
mod run;

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{MarketplaceId, PluginId, UserId};
use systemprompt_web_shared::ProjectId;

use crate::types::{ClientId, FarmAgentId, FarmId};

pub use eval::{EvalCaseRow, EvalDayRow, EvalResultRow, EvalRunRow, EvalSuiteRow};
pub use run::RunRow;

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct FarmRow {
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
    pub synced_commit_sha: Option<String>,
    pub synced_at: Option<DateTime<Utc>>,
    pub synced_by: Option<String>,
    pub sync_error: Option<String>,
    pub project_id: Option<ProjectId>,
    pub owner_user_id: Option<String>,
    pub run_as_user_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub client_id: Option<ClientId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow)]
pub struct AgentRow {
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
    pub updated_at: DateTime<Utc>,
    pub run_as_user_id: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct FixtureRow {
    pub farm_id: FarmId,
    pub name: String,
    pub file_count: i32,
    pub bytes: i64,
    pub stored_path: String,
    pub synced_commit_sha: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct RunsByUserRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub runs: Option<i64>,
    pub succeeded: Option<i64>,
    pub failed: Option<i64>,
    pub active: Option<i64>,
    pub dashboard_runs: Option<i64>,
    pub api_runs: Option<i64>,
    pub eval_runs: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub favourite_agent: Option<String>,
    pub last_run_at: Option<DateTime<Utc>>,
}

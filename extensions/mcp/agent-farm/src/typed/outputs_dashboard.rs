//! Output contract for `farm_dashboard`: the figures the agent-farm dashboard
//! artifact draws. Timestamps are RFC 3339 strings and money is integer
//! microdollars, the same units the admin pages format.

use schemars::JsonSchema;
use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt::mcp::McpOutputSchema;

#[derive(Debug, Clone, Copy, Default, Serialize, JsonSchema)]
pub struct DashboardTotals {
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub success_pct: Option<f64>,
    pub cost_microdollars: i64,
    pub eval_mean: Option<f64>,
    pub completed: i64,
    pub reported: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DashboardClient {
    pub client: String,
    pub name: String,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub active: i64,
    pub people: i64,
    pub success_pct: Option<f64>,
    pub completed: i64,
    pub reported: i64,
    pub eval_mean: Option<f64>,
    pub cost_microdollars: i64,
    pub last_run_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DashboardAgent {
    pub farm: String,
    pub agent: String,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub success_pct: Option<f64>,
    pub completed: i64,
    pub eval_mean: Option<f64>,
    pub cost_microdollars: i64,
    pub last_run_at: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DashboardDay {
    pub day: String,
    pub runs: i64,
    pub succeeded: i64,
    pub finished: i64,
    pub cost_microdollars: i64,
    pub eval_mean: Option<f64>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct DashboardRun {
    pub farm_run_id: String,
    pub farm: String,
    pub agent: String,
    pub client: Option<String>,
    pub client_name: Option<String>,
    pub launched_by: UserId,
    pub status: String,
    pub triggered_via: String,
    pub completed: Option<bool>,
    pub summary: Option<String>,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub cost_microdollars: i64,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FarmDashboardOutput {
    pub scope: String,
    pub window: String,
    pub client: String,
    pub checked_at: String,
    pub totals: DashboardTotals,
    pub clients: Vec<DashboardClient>,
    pub agents: Vec<DashboardAgent>,
    pub daily: Vec<DashboardDay>,
    pub recent_runs: Vec<DashboardRun>,
}

impl McpOutputSchema for FarmDashboardOutput {
    fn artifact_type() -> &'static str {
        "agent_farm_dashboard"
    }
}

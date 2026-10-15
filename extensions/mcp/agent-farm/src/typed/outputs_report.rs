//! Output contracts for a farm run's task-completion report, shared by
//! `run_status` (its `report` field) and `farm_run_report`.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use systemprompt::mcp::McpOutputSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReportOutputFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReportMcpServerCalls {
    pub server: String,
    pub reads: i64,
    pub writes: i64,
    #[serde(default)]
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunReportOutput {
    pub completed: bool,
    pub summary: Option<String>,
    pub final_message: Option<String>,
    pub outputs: Vec<ReportOutputFile>,
    pub mcp_calls: Vec<ReportMcpServerCalls>,
    pub mcp_reads: i32,
    pub mcp_writes: i32,
    pub governance_warnings: i32,
    pub delivery_status: String,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub triggered_via: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub turns: Option<i32>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cost_microdollars: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct FarmRunReportOutput {
    pub run_id: String,
    pub farm_run_id: String,
    pub client: Option<String>,
    pub report: Option<RunReportOutput>,
}

impl McpOutputSchema for FarmRunReportOutput {
    fn artifact_type() -> &'static str {
        "agent_farm_run_report"
    }
}

#[cfg(feature = "farm")]
impl TryFrom<systemprompt_web_admin_farm::repositories::reports::RunReportRow> for RunReportOutput {
    type Error = serde_json::Error;

    fn try_from(
        row: systemprompt_web_admin_farm::repositories::reports::RunReportRow,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            completed: row.completed,
            summary: row.summary,
            final_message: row.final_message,
            outputs: serde_json::from_value(row.outputs)?,
            mcp_calls: serde_json::from_value(row.mcp_calls)?,
            mcp_reads: row.mcp_reads,
            mcp_writes: row.mcp_writes,
            governance_warnings: row.governance_warnings,
            delivery_status: row.delivery_status,
            eval_passed: row.eval_passed,
            eval_score: row.eval_score,
            created_at: row.created_at,
            triggered_via: row.triggered_via,
            started_at: row.started_at,
            finished_at: row.finished_at,
            turns: row.turns,
            input_tokens: row.input_tokens,
            output_tokens: row.output_tokens,
            cost_microdollars: row.cost_microdollars,
        })
    }
}

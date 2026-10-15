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
pub struct ReportOdooWrite {
    pub tool: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub record_ids: Vec<i64>,
    pub decision: String,
    pub policy: String,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RunReportOutput {
    pub completed: bool,
    pub summary: Option<String>,
    pub final_message: Option<String>,
    pub outputs: Vec<ReportOutputFile>,
    pub odoo_writes: Vec<ReportOdooWrite>,
    pub eval_passed: Option<bool>,
    pub eval_score: Option<f64>,
    pub created_at: DateTime<Utc>,
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
            odoo_writes: serde_json::from_value(row.odoo_writes)?,
            eval_passed: row.eval_passed,
            eval_score: row.eval_score,
            created_at: row.created_at,
        })
    }
}

//! The tool list the `agent-farm` server advertises, rendered from the same
//! handler types that execute them, and the schema check the binary runs
//! before it serves.

use rmcp::model::Tool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::McpToolHandler;

use crate::server::{ToolCtx, Viewer};
use crate::typed;

pub const SERVER_NAME: &str = "agent-farm";

pub const TOOL_NAMES: [&str; 16] = [
    "run_start",
    "run_status",
    "run_events",
    "run_cancel",
    "run_list",
    "run_facts",
    "run_stats",
    "farm_run_report",
    "farm_evals_run",
    "farm_eval_results",
    "analytics_costs",
    "analytics_requests",
    "analytics_skills",
    "analytics_tools",
    "catalog_list",
    "farm_dashboard",
];

pub const INSTRUCTIONS: &str = "Agent farm for developers and project managers. Runs: \
    run_start queues a headless agent run on a farm and returns its run id; run_status, \
    run_events and run_facts read one run (status, event stream, recorded facts); run_list \
    and run_stats cover the runs you may see (`scope: mine`, `project`, or `all` for \
    administrators); run_cancel stops one; farm_run_report reads a finished farm run's \
    task-completion report. Evals: farm_evals_run starts a suite scored by \
    deterministic assertions, farm_eval_results reads the verdicts. Analytics: \
    analytics_costs, analytics_requests, analytics_skills, analytics_tools for you or one of \
    your projects over 24h, 7d or 30d. farm_dashboard returns per-client and per-agent \
    outcomes (success rate, task completion, eval score, cost) with recent runs. catalog_list shows what your access grants. You only \
    ever see your own runs and your projects' runs unless you are an administrator.";

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    let ctx = ToolCtx {
        env: None,
        viewer: Viewer::for_listing(),
    };
    let ctx = &ctx;
    let server = McpServerId::new(SERVER_NAME);
    vec![
        typed::RunStartHandler { ctx }.tool_definition(&server),
        typed::RunStatusHandler { ctx }.tool_definition(&server),
        typed::RunEventsHandler { ctx }.tool_definition(&server),
        typed::RunCancelHandler { ctx }.tool_definition(&server),
        typed::RunListHandler { ctx }.tool_definition(&server),
        typed::RunFactsHandler { ctx }.tool_definition(&server),
        typed::RunStatsHandler { ctx }.tool_definition(&server),
        typed::FarmRunReportHandler { ctx }.tool_definition(&server),
        typed::FarmEvalsRunHandler { ctx }.tool_definition(&server),
        typed::FarmEvalResultsHandler { ctx }.tool_definition(&server),
        typed::AnalyticsCostsHandler { ctx }.tool_definition(&server),
        typed::AnalyticsRequestsHandler { ctx }.tool_definition(&server),
        typed::AnalyticsSkillsHandler { ctx }.tool_definition(&server),
        typed::AnalyticsToolsHandler { ctx }.tool_definition(&server),
        typed::CatalogListHandler { ctx }.tool_definition(&server),
        typed::FarmDashboardHandler { ctx }.tool_definition(&server),
    ]
}

#[must_use]
// JSON: rmcp carries each tool schema as a JSON object at the MCP protocol
// boundary.
pub fn schema_violations() -> Vec<systemprompt_mcp_shared::tool_schema::ToolSchemaViolation> {
    list_tools()
        .iter()
        .flat_map(|tool| {
            let input = serde_json::Value::Object((*tool.input_schema).clone());
            let output = tool
                .output_schema
                .as_ref()
                .map(|schema| serde_json::Value::Object((**schema).clone()));
            systemprompt_mcp_shared::tool_schema::tool_violations(
                SERVER_NAME,
                tool.name.as_ref(),
                &input,
                output.as_ref(),
            )
        })
        .collect()
}

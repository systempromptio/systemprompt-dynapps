//! Per-call dispatch: the typed handler for each tool name, run through
//! core's executor so input parsing, output shaping and the tool-usage
//! ledger are the same as every other bundled server's.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use sqlx::PgPool;
use systemprompt::mcp::{ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

use super::viewer::Viewer;
use crate::env::Env;
use crate::error::{ToolError, ToolResult};
use crate::typed;

/// What a handler runs over: the server environment and the resolved caller.
/// The environment is absent only while the tool list is rendered.
#[derive(Debug)]
pub struct ToolCtx<'a> {
    pub env: Option<&'a Env>,
    pub viewer: Viewer,
}

impl ToolCtx<'_> {
    pub fn env(&self) -> ToolResult<&Env> {
        self.env
            .ok_or_else(|| ToolError::Failure("the server environment is not attached".to_owned()))
    }

    pub fn pool(&self) -> ToolResult<&Arc<PgPool>> {
        Ok(&self.env()?.pool)
    }
}

#[derive(Debug)]
pub struct Dispatch<'a> {
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
    pub tool_ctx: &'a ToolCtx<'a>,
}

async fn run<H: McpToolHandler>(
    ctx: &Dispatch<'_>,
    handler: &H,
) -> Result<CallToolResult, McpError> {
    ctx.executor
        .execute(handler, ctx.request, ctx.request_context, ctx.client)
        .await
}

pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    let t = ctx.tool_ctx;
    match tool_name {
        "run_start" => run(ctx, &typed::RunStartHandler { ctx: t }).await,
        "run_status" => run(ctx, &typed::RunStatusHandler { ctx: t }).await,
        "run_events" => run(ctx, &typed::RunEventsHandler { ctx: t }).await,
        "run_cancel" => run(ctx, &typed::RunCancelHandler { ctx: t }).await,
        "run_list" => run(ctx, &typed::RunListHandler { ctx: t }).await,
        "run_facts" => run(ctx, &typed::RunFactsHandler { ctx: t }).await,
        "run_stats" => run(ctx, &typed::RunStatsHandler { ctx: t }).await,
        "farm_run_report" => run(ctx, &typed::FarmRunReportHandler { ctx: t }).await,
        "farm_evals_run" => run(ctx, &typed::FarmEvalsRunHandler { ctx: t }).await,
        "farm_eval_results" => run(ctx, &typed::FarmEvalResultsHandler { ctx: t }).await,
        "analytics_costs" => run(ctx, &typed::AnalyticsCostsHandler { ctx: t }).await,
        "analytics_requests" => run(ctx, &typed::AnalyticsRequestsHandler { ctx: t }).await,
        "analytics_skills" => run(ctx, &typed::AnalyticsSkillsHandler { ctx: t }).await,
        "analytics_tools" => run(ctx, &typed::AnalyticsToolsHandler { ctx: t }).await,
        "catalog_list" => run(ctx, &typed::CatalogListHandler { ctx: t }).await,
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'. This server exposes: {}.",
                crate::tools::TOOL_NAMES.join(", ")
            ),
            None,
        )),
    }
}

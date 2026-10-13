//! Usage over a window for the caller or one of their projects, read from
//! the same analytics repositories the admin dashboard reads. The scope
//! words are resolved in `analytics_scope`.

use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_admin_analytics::repositories::analytics::requests::{
    RequestFilter, get_request_kpis, list_requests_by_model,
};
use systemprompt_web_admin_analytics::repositories::analytics::site::cost::list_provider_costs;
use systemprompt_web_admin_analytics::repositories::analytics::site::kpis::get_site_kpis;
use systemprompt_web_admin_analytics::repositories::analytics::site::skills::list_skill_stats;
use systemprompt_web_admin_analytics::repositories::analytics::site::tools::list_tool_stats;

use super::analytics_scope::resolve;
use super::outputs_analytics::{
    ModelBreakdownOutput, ProviderCostOutput, SkillStatOutput, ToolStatOutput, usd,
};
use super::{
    AnalyticsCostsOutput, AnalyticsInput, AnalyticsRequestsOutput, AnalyticsSkillsOutput,
    AnalyticsToolsOutput, ToolCtx, ToolResult,
};

const PAGE: i64 = 100;

#[derive(Debug, Clone, Copy)]
pub struct AnalyticsCostsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for AnalyticsCostsHandler<'_> {
    type Input = AnalyticsInput;
    type Output = AnalyticsCostsOutput;
    fn tool_name(&self) -> &'static str {
        "analytics_costs"
    }
    fn description(&self) -> &'static str {
        "AI spend over a window for you or one of your projects: requests, errors, tokens, cost in USD, the previous window's cost, and the split by provider."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: AnalyticsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(AnalyticsCostsOutput, String), ErrorData> {
        costs(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn costs(
    ctx: &ToolCtx<'_>,
    input: AnalyticsInput,
) -> ToolResult<(AnalyticsCostsOutput, String)> {
    let (range, site) = resolve(ctx, &input).await?;
    let pool = ctx.pool()?;
    let kpis = get_site_kpis(pool, range, &site).await?;
    let by_provider = list_provider_costs(pool, range, &site)
        .await?
        .into_iter()
        .map(|row| ProviderCostOutput {
            provider: row.label,
            requests: row.requests,
            tokens: row.tokens,
            cost_usd: usd(row.cost_microdollars),
        })
        .collect();
    let output = AnalyticsCostsOutput {
        scope: input.scope,
        window: input.window,
        total_requests: kpis.total_requests,
        error_count: kpis.error_count,
        total_cost_usd: usd(kpis.total_cost_microdollars),
        total_tokens: kpis.total_tokens,
        output_tokens: kpis.output_tokens,
        prev_total_cost_usd: usd(kpis.prev_total_cost_microdollars),
        by_provider,
    };
    let summary = format!(
        "{} requests, ${:.2} over {}",
        output.total_requests, output.total_cost_usd, output.window
    );
    Ok((output, summary))
}

#[derive(Debug, Clone, Copy)]
pub struct AnalyticsRequestsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for AnalyticsRequestsHandler<'_> {
    type Input = AnalyticsInput;
    type Output = AnalyticsRequestsOutput;
    fn tool_name(&self) -> &'static str {
        "analytics_requests"
    }
    fn description(&self) -> &'static str {
        "Gateway requests over a window for you or one of your projects: totals, failures, rejections, governance denials, tool calls, tokens, cost and latency percentiles, with the per-model split."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: AnalyticsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(AnalyticsRequestsOutput, String), ErrorData> {
        requests(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn requests(
    ctx: &ToolCtx<'_>,
    input: AnalyticsInput,
) -> ToolResult<(AnalyticsRequestsOutput, String)> {
    let (range, site) = resolve(ctx, &input).await?;
    let pool = ctx.pool()?;
    let filter = RequestFilter {
        scope: site.scope.clone(),
        user_id: site.user_id.clone(),
        ..RequestFilter::default()
    };
    let kpis = get_request_kpis(pool, &filter, range).await?;
    let by_model = list_requests_by_model(pool, range, &site.scope)
        .await?
        .into_iter()
        .map(|row| ModelBreakdownOutput {
            model: row.key,
            requests: row.requests,
            error_count: row.error_count,
            input_tokens: row.input_tokens,
            output_tokens: row.output_tokens,
            cost_usd: usd(row.cost_microdollars),
            p50_latency_ms: row.p50_latency_ms,
            p95_latency_ms: row.p95_latency_ms,
        })
        .collect();
    let output = AnalyticsRequestsOutput {
        scope: input.scope,
        window: input.window,
        total: kpis.total,
        failed: kpis.failed,
        rejected: kpis.rejected,
        denied: kpis.denied,
        tool_calls: kpis.tool_calls,
        input_tokens: kpis.input_tokens,
        output_tokens: kpis.output_tokens,
        cost_usd: usd(kpis.cost_microdollars),
        p50_latency_ms: kpis.p50_latency_ms,
        p95_latency_ms: kpis.p95_latency_ms,
        by_model,
    };
    let summary = format!("{} requests over {}", output.total, output.window);
    Ok((output, summary))
}

#[derive(Debug, Clone, Copy)]
pub struct AnalyticsSkillsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for AnalyticsSkillsHandler<'_> {
    type Input = AnalyticsInput;
    type Output = AnalyticsSkillsOutput;
    fn tool_name(&self) -> &'static str {
        "analytics_skills"
    }
    fn description(&self) -> &'static str {
        "Skill adoption over a window for you or one of your projects: invocations, people, and the conversations each skill reached with their spend. Per-skill rows overlap; do not sum them."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: AnalyticsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(AnalyticsSkillsOutput, String), ErrorData> {
        skills(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn skills(
    ctx: &ToolCtx<'_>,
    input: AnalyticsInput,
) -> ToolResult<(AnalyticsSkillsOutput, String)> {
    let (range, site) = resolve(ctx, &input).await?;
    let (rows, total) = list_skill_stats(ctx.pool()?, range, &site, PAGE, 0).await?;
    let skills: Vec<SkillStatOutput> = rows
        .into_iter()
        .map(|row| SkillStatOutput {
            skill: row.skill,
            invocations: row.invocations,
            distinct_users: row.distinct_users,
            conversations: row.conversations,
            requests: row.requests,
            conversation_cost_usd: usd(row.conversation_cost_microdollars),
        })
        .collect();
    let returned = skills.len();
    let output = AnalyticsSkillsOutput {
        scope: input.scope,
        window: input.window,
        skills,
        returned,
        total,
        hint: "Rows overlap: a conversation that used two skills counts under both.".to_owned(),
    };
    let summary = format!("{returned} of {total} skills over {}", output.window);
    Ok((output, summary))
}

#[derive(Debug, Clone, Copy)]
pub struct AnalyticsToolsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for AnalyticsToolsHandler<'_> {
    type Input = AnalyticsInput;
    type Output = AnalyticsToolsOutput;
    fn tool_name(&self) -> &'static str {
        "analytics_tools"
    }
    fn description(&self) -> &'static str {
        "MCP tool executions over a window for you or one of your projects: per server and tool, counts by outcome, duration percentiles and distinct users."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: AnalyticsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(AnalyticsToolsOutput, String), ErrorData> {
        tools(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn tools(
    ctx: &ToolCtx<'_>,
    input: AnalyticsInput,
) -> ToolResult<(AnalyticsToolsOutput, String)> {
    let (range, site) = resolve(ctx, &input).await?;
    let (rows, total) = list_tool_stats(ctx.pool()?, range, &site, PAGE, 0).await?;
    let tools: Vec<ToolStatOutput> = rows
        .into_iter()
        .map(|row| ToolStatOutput {
            server: row.server_name,
            tool: row.tool_name,
            executions: row.executions,
            succeeded: row.succeeded,
            failed: row.failed,
            pending: row.pending,
            p50_ms: row.p50_ms,
            p95_ms: row.p95_ms,
            distinct_users: row.distinct_users,
        })
        .collect();
    let returned = tools.len();
    let output = AnalyticsToolsOutput {
        scope: input.scope,
        window: input.window,
        tools,
        returned,
        total,
    };
    let summary = format!("{returned} of {total} tools over {}", output.window);
    Ok((output, summary))
}

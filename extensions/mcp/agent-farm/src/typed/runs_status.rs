//! One run's lifecycle state (`run_status`) and its task-completion report
//! (`farm_run_report`). A farm run's status also carries the client it was
//! for and, once terminal, the report the runner's observer wrote.

use rmcp::ErrorData;
use systemprompt::agent_runs::{AgentRunRow, RunStatus};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

use super::outputs_report::{FarmRunReportOutput, RunReportOutput};
use super::runs_read::load_authorized;
use super::{RunIdInput, RunStatusOutput, ToolCtx, ToolError, ToolResult, run_id};

fn status_output(run: AgentRunRow) -> RunStatusOutput {
    let terminal = run
        .status
        .parse::<RunStatus>()
        .is_ok_and(RunStatus::is_terminal);
    RunStatusOutput {
        run_id: run.id.as_str().to_owned(),
        origin_kind: run.origin_kind,
        origin_id: run.origin_id,
        user_id: run.user_id,
        host: run.host,
        status: run.status,
        terminal,
        error_class: run.error_class,
        error: run.error,
        attempt: run.attempt,
        created_at: run.created_at,
        started_at: run.started_at,
        finished_at: run.finished_at,
        duration_ms: run.duration_ms,
        turns: run.turns,
        input_tokens: run.input_tokens,
        output_tokens: run.output_tokens,
        cost_microdollars: run.cost_microdollars,
        exit_code: run.exit_code,
        cancel_requested: run.cancel_requested_at.is_some(),
        client: None,
        report: None,
    }
}

#[cfg(feature = "farm")]
async fn farm_report(
    ctx: &ToolCtx<'_>,
    run: &AgentRunRow,
) -> ToolResult<Option<(String, Option<String>, Option<RunReportOutput>)>> {
    use systemprompt_web_admin_farm::repositories::reports::find_run_report;
    let Some(farm_run) = super::farm_access::farm_run_of(ctx, run).await? else {
        return Ok(None);
    };
    let report = find_run_report(ctx.pool()?, &farm_run.id)
        .await?
        .map(RunReportOutput::try_from)
        .transpose()?;
    Ok(Some((
        farm_run.id.as_str().to_owned(),
        farm_run.client_id.map(|c| c.as_str().to_owned()),
        report,
    )))
}

#[cfg(not(feature = "farm"))]
async fn farm_report(
    _ctx: &ToolCtx<'_>,
    _run: &AgentRunRow,
) -> ToolResult<Option<(String, Option<String>, Option<RunReportOutput>)>> {
    Ok(None)
}

#[derive(Debug, Clone, Copy)]
pub struct RunStatusHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunStatusHandler<'_> {
    type Input = RunIdInput;
    type Output = RunStatusOutput;
    fn tool_name(&self) -> &'static str {
        "run_status"
    }
    fn description(&self) -> &'static str {
        "One run's lifecycle state: status, timings, tokens, cost, error class, the client it was for, and once terminal its task-completion `report`. Poll this after `run_start` until `terminal` is true. Only your own runs, your projects' runs, or (as an administrator) any run."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunIdInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunStatusOutput, String), ErrorData> {
        status(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn status(ctx: &ToolCtx<'_>, input: &RunIdInput) -> ToolResult<(RunStatusOutput, String)> {
    let id = run_id(&input.run_id)?;
    let run = load_authorized(ctx, &id).await?;
    let farm = farm_report(ctx, &run).await?;
    let mut output = status_output(run);
    if let Some((_, client, report)) = farm {
        output.client = client;
        output.report = report;
    }
    let summary = format!("run {} is {}", output.run_id, output.status);
    Ok((output, summary))
}

#[derive(Debug, Clone, Copy)]
pub struct FarmRunReportHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for FarmRunReportHandler<'_> {
    type Input = RunIdInput;
    type Output = FarmRunReportOutput;
    fn tool_name(&self) -> &'static str {
        "farm_run_report"
    }
    fn description(&self) -> &'static str {
        "The task-completion report of one finished farm run: whether the task completed (run succeeded and its eval, if any, passed), a summary and the final message, the files it wrote (path, size, sha256), how it was triggered (scheduler or manual), its start and end, turns, tokens and cost, its MCP calls per server split into reads and writes, the governance warn findings, whether its summary reached DynBrain, and the eval verdict. `report` is empty while the run is still going."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunIdInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(FarmRunReportOutput, String), ErrorData> {
        report(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn report(
    ctx: &ToolCtx<'_>,
    input: &RunIdInput,
) -> ToolResult<(FarmRunReportOutput, String)> {
    let id = run_id(&input.run_id)?;
    let run = load_authorized(ctx, &id).await?;
    let Some((farm_run_id, client, report)) = farm_report(ctx, &run).await? else {
        return Err(ToolError::Invalid(format!("run {id} is not a farm run")));
    };
    let summary = match &report {
        Some(r) if r.completed => format!("run {id} completed"),
        Some(_) => format!("run {id} did not complete"),
        None => format!("run {id} has no report yet ({})", run.status),
    };
    Ok((
        FarmRunReportOutput {
            run_id: id.as_str().to_owned(),
            farm_run_id,
            client,
            report,
        },
        summary,
    ))
}

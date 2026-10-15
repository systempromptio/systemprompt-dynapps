//! The two tools that change a run: queue one, and ask one to stop.
//!
//! `run_start` goes through the farm's own launch service, so limits,
//! prompts, credentials and the `agent_farm_runs` row are resolved exactly as
//! the dashboard resolves them; it never waits for the run. With a `case` it
//! is also an eval of that case, judged by the farms file's configured
//! judge once the run finishes. `run_cancel`
//! goes through the farm's cancel for a farm run, and straight to core's row
//! for any other origin.

use rmcp::ErrorData;
use systemprompt::agent_runs::{AgentRunRow, RunStatus};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

use super::{
    RunCancelOutput, RunIdInput, RunStartInput, RunStartOutput, ToolCtx, ToolError, ToolResult,
    run_id,
};

#[derive(Debug, Clone, Copy)]
pub struct RunStartHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunStartHandler<'_> {
    type Input = RunStartInput;
    type Output = RunStartOutput;
    fn tool_name(&self) -> &'static str {
        "run_start"
    }
    fn description(&self) -> &'static str {
        "Queue one headless agent run on a farm and return at once with its run id. The run executes under the farm's limits unless you set tighter ones; poll `run_status`, read `run_events`. Name a `client` slug to attribute the run to a client other than the farm's. Name an eval `case` to have the run evaluated when it finishes (`eval_run_id` in the result; read it with `farm_eval_results`). You must hold the farm's workspace."
    }
    async fn handle(
        &self,
        input: RunStartInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunStartOutput, String), ErrorData> {
        start(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn start(ctx: &ToolCtx<'_>, input: RunStartInput) -> ToolResult<(RunStartOutput, String)> {
    match input.origin.trim() {
        "" | "farm" => start_farm_run(ctx, input).await,
        "workflow" => Err(workflow_unavailable()),
        other => Err(ToolError::Invalid(format!(
            "`origin` must be `farm` (got {other:?})"
        ))),
    }
}

#[cfg(feature = "workflows")]
fn workflow_unavailable() -> ToolError {
    ToolError::Invalid(
        "`origin: workflow` runs are started by the workflow engine, not by this tool".to_owned(),
    )
}

#[cfg(not(feature = "workflows"))]
fn workflow_unavailable() -> ToolError {
    ToolError::Invalid(
        "`origin: workflow` is not available on this instance; use `origin: farm`".to_owned(),
    )
}

#[cfg(feature = "farm")]
fn check_run_project(
    ctx: &ToolCtx<'_>,
    farm: &systemprompt_web_admin_farm::repositories::rows::FarmRow,
    project: &str,
) -> ToolResult<()> {
    if !project.is_empty() && !ctx.viewer.is_admin && !ctx.viewer.is_member_of(project) {
        return Err(ToolError::Forbidden(format!(
            "you are not a member of project {project}"
        )));
    }
    if !project.is_empty()
        && farm
            .project_id
            .as_ref()
            .is_some_and(|own| own.as_str() != project)
    {
        return Err(ToolError::Invalid(format!(
            "farm {} belongs to project {}, not {project}; a farm run is attributed to its farm's project",
            farm.id,
            farm.project_id.as_ref().map_or("", |p| p.as_str())
        )));
    }
    Ok(())
}

#[cfg(feature = "farm")]
async fn start_farm_run(
    ctx: &ToolCtx<'_>,
    input: RunStartInput,
) -> ToolResult<(RunStartOutput, String)> {
    use std::sync::Arc;
    use systemprompt_web_admin_farm::services::evals::{CaseLaunch, start_case_eval};
    use systemprompt_web_admin_farm::services::runs::{LaunchRequest, launch, resolve_client};
    use systemprompt_web_admin_farm::types::{FarmAgentId, TriggeredVia};

    let farm = super::farm_access::entitled_farm(ctx, &input.farm).await?;
    check_run_project(ctx, &farm, input.project.trim())?;
    let state = ctx.env()?.farm_state()?;
    let client_id = resolve_client(&state.pool, Some(input.client.as_str())).await?;
    let max_turns =
        (input.max_turns > 0).then(|| i32::try_from(input.max_turns).unwrap_or(i32::MAX));
    let timeout_seconds = (input.timeout_seconds > 0)
        .then(|| i32::try_from(input.timeout_seconds).unwrap_or(i32::MAX));
    let request = LaunchRequest {
        farm_id: farm.id,
        agent_id: FarmAgentId::new(super::non_empty("agent", &input.agent)?),
        user_id: ctx.viewer.user_id.clone(),
        triggered_via: TriggeredVia::Mcp,
        prompt: Some(input.prompt).filter(|p| !p.trim().is_empty()),
        case_prompt: None,
        fixture: None,
        model: None,
        max_turns,
        max_budget_usd: (input.max_budget_usd > 0.0).then_some(input.max_budget_usd),
        timeout_seconds,
        eval_result_id: None,
        client_id,
    };
    let case_id = input.case.trim();
    let (row, eval_run_id) = if case_id.is_empty() {
        (launch(state, request).await?, None)
    } else {
        let judge = state.judge.judge_for(None, &ctx.viewer.user_id);
        let launched = start_case_eval(
            Arc::clone(state),
            judge,
            CaseLaunch {
                request,
                case_id: case_id.to_owned(),
            },
        )
        .await?;
        (launched.run, Some(launched.eval_run.id.as_str().to_owned()))
    };
    let summary = format!(
        "queued run {} on {}/{}",
        row.agent_run_id, row.farm_id, row.agent_id
    );
    Ok((
        RunStartOutput {
            run_id: row.agent_run_id.as_str().to_owned(),
            origin: "farm".to_owned(),
            farm_run_id: row.id.as_str().to_owned(),
            status: row.status,
            eval_run_id,
            hint: "Poll `run_status` until `terminal`; `run_events` streams the transcript."
                .to_owned(),
        },
        summary,
    ))
}

#[cfg(not(feature = "farm"))]
async fn start_farm_run(
    _ctx: &ToolCtx<'_>,
    _input: RunStartInput,
) -> ToolResult<(RunStartOutput, String)> {
    Err(ToolError::Invalid(
        "`origin: farm` is not available on this instance".to_owned(),
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct RunCancelHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunCancelHandler<'_> {
    type Input = RunIdInput;
    type Output = RunCancelOutput;
    fn tool_name(&self) -> &'static str {
        "run_cancel"
    }
    fn description(&self) -> &'static str {
        "Ask a run to stop. A queued run is cancelled at once; a running one is stopped by its runner within a few seconds. Returns `cancelled: false` when the run had already finished."
    }
    async fn handle(
        &self,
        input: RunIdInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunCancelOutput, String), ErrorData> {
        cancel_run(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn cancel_run(
    ctx: &ToolCtx<'_>,
    input: &RunIdInput,
) -> ToolResult<(RunCancelOutput, String)> {
    let id = run_id(&input.run_id)?;
    let run = super::runs_read::load_authorized(ctx, &id).await?;
    let (farm_run_id, cancelled) = cancel(ctx, &run).await?;
    let message = if cancelled {
        "cancellation requested".to_owned()
    } else {
        format!("run already {}", run.status)
    };
    let summary = format!("run {id}: {message}");
    Ok((
        RunCancelOutput {
            run_id: id.as_str().to_owned(),
            farm_run_id,
            cancelled,
            message,
        },
        summary,
    ))
}

async fn cancel(ctx: &ToolCtx<'_>, run: &AgentRunRow) -> ToolResult<(String, bool)> {
    #[cfg(feature = "farm")]
    if let Some(farm_run) = super::farm_access::farm_run_of(ctx, run).await? {
        let state = ctx.env()?.farm_state()?;
        let flagged = systemprompt_web_admin_farm::services::runs::cancel(
            state,
            &farm_run.id,
            &ctx.viewer.user_id,
        )
        .await?;
        return Ok((farm_run.id.as_str().to_owned(), flagged));
    }
    let repo = super::runs_read::repo(ctx)?;
    if run
        .status
        .parse::<RunStatus>()
        .is_ok_and(RunStatus::is_terminal)
    {
        return Ok((String::new(), false));
    }
    if !repo.cancel_queued(&run.id).await? {
        repo.update_cancel_requested(&run.id).await?;
    }
    Ok((String::new(), true))
}

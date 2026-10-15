//! The read-only run tools: one run's status, its events, the list a
//! caller may see, and the recorded facts of one run. Rollups and the
//! scope words are in `runs_stats`.

use std::sync::Arc;

use rmcp::ErrorData;
use systemprompt::agent_runs::{AgentRunRepository, AgentRunRow};
use systemprompt::identifiers::{AgentRunId, McpExecutionId};
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_admin_core::repositories::agent_run_facts::{
    self as facts, RunFacts, RunFilter,
};

use super::outputs::RunEventOutput;
use super::{
    MAX_EVENTS, MAX_LIST, RunEventsInput, RunEventsOutput, RunExtras, RunFactsOutput, RunIdInput,
    RunListInput, RunListOutput, ToolCtx, ToolError, ToolResult, bounded, facts_scope, run_id,
};
use crate::server::{RunOrigin, authorize_run};

pub(crate) fn repo(ctx: &ToolCtx<'_>) -> ToolResult<AgentRunRepository> {
    Ok(AgentRunRepository::from_pool(Arc::clone(ctx.pool()?)))
}

pub(crate) async fn load_run(ctx: &ToolCtx<'_>, id: &AgentRunId) -> ToolResult<AgentRunRow> {
    repo(ctx)?
        .find_run(id)
        .await?
        .ok_or_else(|| ToolError::NotFound(format!("run {id} does not exist")))
}

pub(crate) async fn load_facts(ctx: &ToolCtx<'_>, id: &AgentRunId) -> ToolResult<Option<RunFacts>> {
    Ok(facts::find_run_facts(ctx.pool()?, id).await?)
}

pub(crate) async fn origin_of(ctx: &ToolCtx<'_>, run: &AgentRunRow) -> ToolResult<RunOrigin> {
    let mut origin = RunOrigin::default();
    if let Some(found) = load_facts(ctx, &run.id).await? {
        origin.project_id = found.project_id;
        origin.launched_by = Some(found.user_id);
    }
    #[cfg(feature = "farm")]
    if let Some(farm_run) = super::farm_access::farm_run_of(ctx, run).await? {
        origin.launched_by = Some(farm_run.user_id);
    }
    Ok(origin)
}

pub(crate) async fn load_authorized(ctx: &ToolCtx<'_>, id: &AgentRunId) -> ToolResult<AgentRunRow> {
    let run = load_run(ctx, id).await?;
    let origin = origin_of(ctx, &run).await?;
    authorize_run(&ctx.viewer, &run, &origin)?;
    Ok(run)
}

#[derive(Debug, Clone, Copy)]
pub struct RunEventsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunEventsHandler<'_> {
    type Input = RunEventsInput;
    type Output = RunEventsOutput;
    fn tool_name(&self) -> &'static str {
        "run_events"
    }
    fn description(&self) -> &'static str {
        "A page of one run's event stream (init, assistant text, tool use and result, hook decisions, final result), ordered by `seq`. Pass the returned `next_after_seq` as `after_seq` to continue."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunEventsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunEventsOutput, String), ErrorData> {
        events(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn events(
    ctx: &ToolCtx<'_>,
    input: &RunEventsInput,
) -> ToolResult<(RunEventsOutput, String)> {
    let id = run_id(&input.run_id)?;
    load_authorized(ctx, &id).await?;
    let limit = bounded(input.limit, 200, MAX_EVENTS);
    let repo = repo(ctx)?;
    let rows = repo
        .list_events_after(&id, input.after_seq.max(-1), limit)
        .await?;
    let total_events = repo.count_events(&id).await?;
    let next_after_seq = rows.last().map_or(input.after_seq, |e| e.seq);
    let events = rows
        .into_iter()
        .map(|e| {
            Ok(RunEventOutput {
                seq: e.seq,
                kind: e.kind.as_str().to_owned(),
                tool_name: e.tool_name,
                tool_use_id: e.tool_use_id,
                created_at: e.created_at,
                payload: serde_json::to_value(&e.payload)?,
            })
        })
        .collect::<ToolResult<Vec<RunEventOutput>>>()?;
    let returned = events.len();
    let exhausted = (returned as i64) < limit;
    let summary = format!("{returned} events of {total_events} for run {id}");
    Ok((
        RunEventsOutput {
            run_id: id.as_str().to_owned(),
            events,
            returned,
            total_events,
            next_after_seq,
            exhausted,
        },
        summary,
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct RunListHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunListHandler<'_> {
    type Input = RunListInput;
    type Output = RunListOutput;
    fn tool_name(&self) -> &'static str {
        "run_list"
    }
    fn description(&self) -> &'static str {
        "Runs you may see, newest first, each with its recorded facts (status, cost, tokens, tool counts, hook and governance decisions). `scope: mine` is your own runs, `project` every run of your projects, `all` the whole instance (administrators only)."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunListInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunListOutput, String), ErrorData> {
        list(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn list(ctx: &ToolCtx<'_>, input: RunListInput) -> ToolResult<(RunListOutput, String)> {
    let scope = facts_scope(&ctx.viewer, &input.scope)?;
    let filter = RunFilter {
        origin_kind: Some(input.origin.trim().to_owned()).filter(|s| !s.is_empty()),
        status: Some(input.state.trim().to_owned()).filter(|s| !s.is_empty()),
        limit: Some(bounded(input.limit, 50, MAX_LIST)),
        ..RunFilter::default()
    };
    let runs = facts::list_run_facts(ctx.pool()?, &scope, &filter).await?;
    let returned = runs.len();
    let summary = format!("{returned} runs in scope {}", input.scope);
    Ok((
        RunListOutput {
            scope: input.scope,
            runs,
            returned,
            hint: "Use `run_facts` for one run's full record, `run_events` for its stream."
                .to_owned(),
        },
        summary,
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct RunFactsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunFactsHandler<'_> {
    type Input = RunIdInput;
    type Output = RunFactsOutput;
    fn tool_name(&self) -> &'static str {
        "run_facts"
    }
    fn description(&self) -> &'static str {
        "Everything the tables recorded about one run: outcome, tokens, reported and metered cost, tool use counts by name, hook and governance decisions, structured output and whether it matched its schema, the diff summary, and the origin's own record (for a farm run: the agent, prompt and any deterministic assertion results). No model judges anything."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunIdInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunFactsOutput, String), ErrorData> {
        facts_of(self.ctx, &input).await.map_err(ErrorData::from)
    }
}

async fn facts_of(ctx: &ToolCtx<'_>, input: &RunIdInput) -> ToolResult<(RunFactsOutput, String)> {
    let id = run_id(&input.run_id)?;
    let run = load_authorized(ctx, &id).await?;
    let facts = load_facts(ctx, &id)
        .await?
        .ok_or_else(|| ToolError::NotFound(format!("run {id} has no recorded facts yet")))?;
    let extras = extras_for(ctx, &run).await?;
    let summary = format!("run {} {} ({})", id, facts.status, run.origin_kind);
    Ok((RunFactsOutput { facts, extras }, summary))
}

#[cfg(feature = "farm")]
async fn extras_for(ctx: &ToolCtx<'_>, run: &AgentRunRow) -> ToolResult<RunExtras> {
    Ok(super::farm_access::farm_extras(ctx, run)
        .await?
        .unwrap_or(RunExtras::None))
}

#[cfg(not(feature = "farm"))]
async fn extras_for(_ctx: &ToolCtx<'_>, _run: &AgentRunRow) -> ToolResult<RunExtras> {
    Ok(RunExtras::None)
}

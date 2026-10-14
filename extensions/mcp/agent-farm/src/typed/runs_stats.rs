//! Run rollups over a window, and the scope words every run tool shares:
//!
//! `mine` is the caller's own runs, `project` every run of the projects they
//! belong to, `all` the instance (administrators only), and for the stats
//! alone `farm` one farm's runs.

use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_admin_core::repositories::agent_run_facts::{
    self as facts, RunFilter, RunScope, RunStats,
};
use systemprompt_web_shared::ProjectId;

use super::{RunStatsInput, RunStatsOutput, ToolCtx, ToolError, ToolResult, window};
use crate::server::Viewer;

pub fn facts_scope(viewer: &Viewer, scope: &str) -> ToolResult<RunScope> {
    match scope.trim() {
        "" | "mine" => Ok(RunScope::Own(viewer.user_id.clone())),
        "project" => {
            if viewer.project_ids.is_empty() {
                return Err(ToolError::Invalid(
                    "you belong to no project; use `scope: mine`".to_owned(),
                ));
            }
            Ok(RunScope::Projects(
                viewer
                    .project_ids
                    .iter()
                    .map(|id| ProjectId::new(id.clone()))
                    .collect(),
            ))
        },
        "all" => {
            if viewer.is_admin {
                Ok(RunScope::All)
            } else {
                Err(ToolError::Forbidden(
                    "`scope: all` spans every user and needs an administrator role".to_owned(),
                ))
            }
        },
        other => Err(ToolError::Invalid(format!(
            "`scope` must be `mine`, `project` or `all` (got {other:?})"
        ))),
    }
}

#[cfg(feature = "farm")]
async fn farm_stats(ctx: &ToolCtx<'_>, farm: &str, filter: &mut RunFilter) -> ToolResult<RunStats> {
    let row = super::farm_access::entitled_farm(ctx, farm).await?;
    filter.farm_id = Some(row.id.as_str().to_owned());
    let pool = ctx.pool()?;
    if ctx.viewer.is_admin {
        return Ok(facts::get_agent_run_stats(pool, &RunScope::All, filter).await?);
    }
    let own = RunScope::Own(ctx.viewer.user_id.clone());
    if ctx.viewer.project_ids.is_empty() {
        return Ok(facts::get_agent_run_stats(pool, &own, filter).await?);
    }
    let projects = facts_scope(&ctx.viewer, "project")?;
    let stats = facts::get_agent_run_stats(pool, &projects, filter).await?;
    if stats.totals.total > 0 {
        return Ok(stats);
    }
    Ok(facts::get_agent_run_stats(pool, &own, filter).await?)
}

#[cfg(not(feature = "farm"))]
async fn farm_stats(
    _ctx: &ToolCtx<'_>,
    _farm: &str,
    _filter: &mut RunFilter,
) -> ToolResult<RunStats> {
    Err(ToolError::Invalid(
        "`scope: farm` is not available on this instance".to_owned(),
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct RunStatsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for RunStatsHandler<'_> {
    type Input = RunStatsInput;
    type Output = RunStatsOutput;
    fn tool_name(&self) -> &'static str {
        "run_stats"
    }
    fn description(&self) -> &'static str {
        "Run rollups over a window: totals by status and error class, cost, duration percentiles, tool use and hook denials, split by origin kind. `scope: farm` needs `farm` and covers that farm's runs you may see (your projects' runs, else your own); `all` is administrators only."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: RunStatsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(RunStatsOutput, String), ErrorData> {
        stats(self.ctx, input).await.map_err(ErrorData::from)
    }
}

async fn stats(ctx: &ToolCtx<'_>, input: RunStatsInput) -> ToolResult<(RunStatsOutput, String)> {
    let range = window(&input.window)?;
    let mut filter = RunFilter {
        since: Some(range.from),
        until: Some(range.to),
        ..RunFilter::default()
    };
    let stats = if input.scope.trim() == "farm" {
        let farm = super::non_empty("farm", &input.farm)?;
        farm_stats(ctx, &farm, &mut filter).await?
    } else {
        let scope = facts_scope(&ctx.viewer, &input.scope)?;
        facts::get_agent_run_stats(ctx.pool()?, &scope, &filter).await?
    };
    let summary = format!(
        "{} runs in scope {} over {}",
        stats.totals.total, input.scope, input.window
    );
    Ok((
        RunStatsOutput {
            scope: input.scope,
            window: input.window,
            farm: input.farm,
            stats,
        },
        summary,
    ))
}

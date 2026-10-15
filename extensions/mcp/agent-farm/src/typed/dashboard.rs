//! `farm_dashboard`: the per-client and per-agent outcome rollups, the daily
//! series and the recent runs with their completion report and eval verdict
//! that the agent-farm dashboard artifact draws. The figures come from the
//! same farm repository queries as `/admin/farm/clients`, so the artifact and
//! the admin pages cannot disagree. `all` is administrators only; anyone else
//! sees the runs they launched.

use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

use super::outputs_dashboard::FarmDashboardOutput;
use super::{FarmDashboardInput, MAX_LIST, ToolCtx, ToolError, ToolResult, bounded, window};

#[derive(Debug, Clone, Copy)]
pub struct FarmDashboardHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for FarmDashboardHandler<'_> {
    type Input = FarmDashboardInput;
    type Output = FarmDashboardOutput;
    fn tool_name(&self) -> &'static str {
        "farm_dashboard"
    }
    fn description(&self) -> &'static str {
        "Farm outcomes over a window for the dashboard: totals, one row per client and per agent (runs, success rate, task completion, mean eval score, cost), a zero-filled daily series, and recent runs with their completion report and eval verdict. `scope: all` is administrators only; otherwise your own runs. `client` narrows to one client id."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: FarmDashboardInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(FarmDashboardOutput, String), ErrorData> {
        dashboard(self.ctx, input).await.map_err(ErrorData::from)
    }
}

fn resolve_scope(ctx: &ToolCtx<'_>, scope: &str) -> ToolResult<&'static str> {
    match (scope.trim(), ctx.viewer.is_admin) {
        ("" | "all", true) => Ok("all"),
        ("", false) | ("mine", _) => Ok("mine"),
        ("all", false) => Err(ToolError::Forbidden(
            "`scope: all` spans every user and needs an administrator role".to_owned(),
        )),
        (other, _) => Err(ToolError::Invalid(format!(
            "`scope` must be `mine` or `all` (got {other:?})"
        ))),
    }
}

#[cfg(feature = "farm")]
async fn dashboard(
    ctx: &ToolCtx<'_>,
    input: FarmDashboardInput,
) -> ToolResult<(FarmDashboardOutput, String)> {
    use super::outputs_dashboard::DashboardTotals;
    use systemprompt_web_admin_farm::repositories::clients::{
        ReportScope, list_agent_outcomes, list_client_summaries, list_daily_outcomes,
        list_report_runs,
    };
    use systemprompt_web_admin_farm::types::ClientId;

    let scope = resolve_scope(ctx, &input.scope)?;
    let range = window(&input.window)?;
    let user = (scope == "mine").then(|| ctx.viewer.user_id.clone());
    let client = input.client.trim().to_owned();
    let client_key = (!client.is_empty()).then(|| ClientId::new(client.clone()));
    let report = ReportScope {
        client: client_key.as_ref(),
        user: user.as_ref(),
        since: range.from,
    };
    let pool = ctx.pool()?;
    let limit = bounded(input.limit, 50, MAX_LIST);
    let (clients, agents, daily, runs) = tokio::join!(
        list_client_summaries(pool, report),
        list_agent_outcomes(pool, report),
        list_daily_outcomes(pool, report),
        list_report_runs(pool, report.client, report.user, limit),
    );
    let (clients, agents, daily, runs) = (clients?, agents?, daily?, runs?);
    let scores: Vec<f64> = daily.iter().filter_map(|d| d.eval_mean).collect();
    let mut totals = DashboardTotals {
        runs: daily.iter().map(|d| d.runs).sum(),
        succeeded: daily.iter().map(|d| d.succeeded).sum(),
        finished: daily.iter().map(|d| d.finished).sum(),
        cost_microdollars: daily.iter().map(|d| d.cost_microdollars).sum(),
        eval_mean: (!scores.is_empty()).then(|| scores.iter().sum::<f64>() / scores.len() as f64),
        completed: clients.iter().map(|c| c.completed).sum(),
        reported: clients.iter().map(|c| c.reported).sum(),
        success_pct: None,
    };
    totals.success_pct = convert::pct(totals.succeeded, totals.finished);
    let summary = format!(
        "{} runs across {} clients in scope {scope} over {}",
        totals.runs,
        clients.iter().filter(|c| c.runs > 0).count(),
        input.window
    );
    Ok((
        FarmDashboardOutput {
            scope: scope.to_owned(),
            window: input.window,
            client,
            checked_at: chrono::Utc::now().to_rfc3339(),
            totals,
            clients: clients.iter().map(convert::client).collect(),
            agents: agents.iter().map(convert::agent).collect(),
            daily: daily.iter().map(convert::day).collect(),
            recent_runs: runs.iter().map(convert::run).collect(),
        },
        summary,
    ))
}

#[cfg(not(feature = "farm"))]
async fn dashboard(
    ctx: &ToolCtx<'_>,
    input: FarmDashboardInput,
) -> ToolResult<(FarmDashboardOutput, String)> {
    resolve_scope(ctx, &input.scope)?;
    Err(ToolError::Invalid(
        "`farm_dashboard` is not available on this instance".to_owned(),
    ))
}

#[cfg(feature = "farm")]
mod convert {
    use systemprompt_web_admin_farm::repositories::clients::{
        AgentOutcomeRow, ClientSummaryRow, DailyOutcomeRow, ReportRunRow,
    };

    use super::super::outputs_dashboard::{
        DashboardAgent, DashboardClient, DashboardDay, DashboardRun,
    };

    pub(super) fn pct(part: i64, whole: i64) -> Option<f64> {
        (whole > 0).then(|| part as f64 * 100.0 / whole as f64)
    }

    pub(super) fn client(r: &ClientSummaryRow) -> DashboardClient {
        DashboardClient {
            client: r.client_id.as_str().to_owned(),
            name: r.name.clone(),
            runs: r.runs,
            succeeded: r.succeeded,
            finished: r.finished,
            active: r.active,
            people: r.people,
            success_pct: pct(r.succeeded, r.finished),
            completed: r.completed,
            reported: r.reported,
            eval_mean: r.eval_mean,
            cost_microdollars: r.cost_microdollars,
            last_run_at: r.last_run_at.map(|t| t.to_rfc3339()),
        }
    }

    pub(super) fn agent(r: &AgentOutcomeRow) -> DashboardAgent {
        DashboardAgent {
            farm: r.farm_id.as_str().to_owned(),
            agent: r.agent_id.as_str().to_owned(),
            runs: r.runs,
            succeeded: r.succeeded,
            finished: r.finished,
            success_pct: pct(r.succeeded, r.finished),
            completed: r.completed,
            eval_mean: r.eval_mean,
            cost_microdollars: r.cost_microdollars,
            last_run_at: r.last_run_at.to_rfc3339(),
        }
    }

    pub(super) fn day(r: &DailyOutcomeRow) -> DashboardDay {
        DashboardDay {
            day: r.day.to_string(),
            runs: r.runs,
            succeeded: r.succeeded,
            finished: r.finished,
            cost_microdollars: r.cost_microdollars,
            eval_mean: r.eval_mean,
        }
    }

    pub(super) fn run(r: &ReportRunRow) -> DashboardRun {
        DashboardRun {
            farm_run_id: r.run_id.as_str().to_owned(),
            farm: r.farm_id.as_str().to_owned(),
            agent: r.agent_id.as_str().to_owned(),
            client: r.client_id.as_ref().map(|c| c.as_str().to_owned()),
            client_name: r.client_name.clone(),
            launched_by: r.user_id.clone(),
            status: r.status.clone(),
            triggered_via: r.triggered_via.clone(),
            completed: r.completed,
            summary: r.summary.clone(),
            eval_passed: r.eval_passed,
            eval_score: r.eval_score,
            cost_microdollars: r.cost_microdollars,
            created_at: r.created_at.to_rfc3339(),
            finished_at: r.finished_at.map(|t| t.to_rfc3339()),
        }
    }
}

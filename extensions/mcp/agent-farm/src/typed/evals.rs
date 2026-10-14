//! Farm evals: start a suite, read the latest verdict per case, and read the
//! eval history. A suite started here is judged by the farms file's
//! configured judge, the same one the public API and the console build.

use rmcp::ErrorData;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

use super::{
    EvalResultsOutput, EvalRunOutput, FarmEvalResultsInput, FarmEvalsRunInput, ToolCtx, ToolResult,
};

#[derive(Debug, Clone, Copy)]
pub struct FarmEvalsRunHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for FarmEvalsRunHandler<'_> {
    type Input = FarmEvalsRunInput;
    type Output = EvalRunOutput;
    fn tool_name(&self) -> &'static str {
        "farm_evals_run"
    }
    fn description(&self) -> &'static str {
        "Start an eval suite on a farm: one headless run per case, each scored by the suite's deterministic assertions (file exists, output matches, tool was or was not used) and, for a case with a rubric, by the configured judge model. Returns the eval run id at once; read results with `farm_eval_results` or per run with `run_facts`."
    }
    async fn handle(
        &self,
        input: FarmEvalsRunInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(EvalRunOutput, String), ErrorData> {
        start_eval(self.ctx, input).await.map_err(ErrorData::from)
    }
}

#[cfg(feature = "farm")]
async fn start_eval(
    ctx: &ToolCtx<'_>,
    input: FarmEvalsRunInput,
) -> ToolResult<(EvalRunOutput, String)> {
    use std::sync::Arc;
    use systemprompt_web_admin_farm::services::evals::{EvalLaunch, start_eval_run};
    use systemprompt_web_admin_farm::types::{EvalSuiteId, TriggeredVia};

    let farm = super::farm_access::entitled_farm(ctx, &input.farm).await?;
    let suite_id = EvalSuiteId::new(super::non_empty("suite", &input.suite)?);
    let case_filter: Vec<String> = input
        .case_filter
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let state = ctx.env()?.farm_state()?;
    let judge = state.judge.judge_for(None, &ctx.viewer.user_id);
    let row = start_eval_run(
        Arc::clone(state),
        judge,
        EvalLaunch {
            farm_id: farm.id,
            suite_id,
            user_id: ctx.viewer.user_id.clone(),
            triggered_via: TriggeredVia::Mcp,
            case_filter: (!case_filter.is_empty()).then_some(case_filter),
        },
    )
    .await?;
    let summary = format!("eval run {} started on {}", row.id, row.suite_id);
    Ok((
        EvalRunOutput {
            eval_run_id: row.id.as_str().to_owned(),
            farm: row.farm_id.as_str().to_owned(),
            suite: row.suite_id.as_str().to_owned(),
            agent: row.agent_id.as_str().to_owned(),
            status: row.status,
            hint: "Each case is a run; `run_list` shows them, `farm_eval_results` the verdicts."
                .to_owned(),
        },
        summary,
    ))
}

#[cfg(not(feature = "farm"))]
async fn start_eval(
    _ctx: &ToolCtx<'_>,
    _input: FarmEvalsRunInput,
) -> ToolResult<(EvalRunOutput, String)> {
    Err(super::ToolError::Invalid(
        "farm evals are not available on this instance".to_owned(),
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct FarmEvalResultsHandler<'a> {
    pub ctx: &'a ToolCtx<'a>,
}

impl McpToolHandler for FarmEvalResultsHandler<'_> {
    type Input = FarmEvalResultsInput;
    type Output = EvalResultsOutput;
    fn tool_name(&self) -> &'static str {
        "farm_eval_results"
    }
    fn description(&self) -> &'static str {
        "The latest decided verdict for every (suite, case) on the farms you may use: passed, failed or errored, with the eval run that decided it. Set `since` (`7d`, `24h`, an RFC 3339 time) to also get `history`: per agent per day eval runs, case pass rate, mean judge score and cost."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: FarmEvalResultsInput,
        _context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(EvalResultsOutput, String), ErrorData> {
        eval_results(self.ctx, input).await.map_err(ErrorData::from)
    }
}

#[cfg(feature = "farm")]
async fn eval_results(
    ctx: &ToolCtx<'_>,
    input: FarmEvalResultsInput,
) -> ToolResult<(EvalResultsOutput, String)> {
    use std::collections::BTreeMap;
    use systemprompt_web_admin_farm::repositories::{evals, farms};
    use systemprompt_web_admin_farm::services::entitlement::may_use_farm;
    use systemprompt_web_admin_farm::types::FarmId;

    use super::outputs::EvalCellOutput;

    let history = eval_history(ctx, &input).await?;

    let pool = ctx.pool()?;
    let mut usable: Vec<FarmId> = Vec::new();
    if input.farm.trim().is_empty() {
        for farm in farms::list_farms(pool).await? {
            if may_use_farm(pool, ctx.viewer.context()?, &farm).await? {
                usable.push(farm.id);
            }
        }
    } else {
        usable.push(
            super::farm_access::entitled_farm(ctx, &input.farm)
                .await?
                .id,
        );
    }
    let suite_farms: BTreeMap<String, String> = evals::list_all_eval_suites(pool)
        .await?
        .into_iter()
        .filter(|s| usable.contains(&s.farm_id))
        .map(|s| (s.id.as_str().to_owned(), s.farm_id.as_str().to_owned()))
        .collect();
    let one_farm = (usable.len() == 1).then(|| usable[0].clone());
    let suite = input.suite.trim();
    let rows: Vec<EvalCellOutput> = evals::list_latest_results_matrix(pool, one_farm.as_ref())
        .await?
        .into_iter()
        .filter(|cell| suite.is_empty() || cell.suite_id.as_str() == suite)
        .filter_map(|cell| {
            let farm = suite_farms.get(cell.suite_id.as_str())?.clone();
            Some(EvalCellOutput {
                farm,
                suite: cell.suite_id.as_str().to_owned(),
                case_id: cell.case_id,
                agent: cell.agent_id.as_str().to_owned(),
                eval_run_id: cell.eval_run_id.as_str().to_owned(),
                status: cell.status,
                passed: cell.passed,
                finished_at: cell.finished_at,
            })
        })
        .collect();
    let count = |status: &str| rows.iter().filter(|r| r.status == status).count();
    let output = EvalResultsOutput {
        returned: rows.len(),
        passed: count("passed"),
        failed: count("failed"),
        errored: count("errored"),
        rows,
        history,
    };
    let summary = format!(
        "{} cases: {} passed, {} failed, {} errored",
        output.returned, output.passed, output.failed, output.errored
    );
    Ok((output, summary))
}

#[cfg(feature = "farm")]
async fn eval_history(
    ctx: &ToolCtx<'_>,
    input: &FarmEvalResultsInput,
) -> ToolResult<Vec<super::outputs::EvalDayOutput>> {
    use systemprompt_web_admin_farm::services::evals::{list_visible_history, parse_since};
    use systemprompt_web_admin_farm::types::FarmId;

    use super::outputs::EvalDayOutput;

    if input.since.trim().is_empty() {
        return Ok(Vec::new());
    }
    let since = parse_since(&input.since, chrono::Utc::now()).ok_or_else(|| {
        super::ToolError::Invalid(format!(
            "`since` {:?} is not `<n>d`, `<n>h` or an RFC 3339 time",
            input.since
        ))
    })?;
    let farm = Some(input.farm.trim())
        .filter(|f| !f.is_empty())
        .map(FarmId::new);
    let points = list_visible_history(
        ctx.pool()?,
        ctx.viewer.context()?,
        farm.as_ref(),
        None,
        since,
    )
    .await?;
    Ok(points
        .into_iter()
        .map(|p| EvalDayOutput {
            farm: p.farm.as_str().to_owned(),
            agent: p.agent.as_str().to_owned(),
            day: p.day.to_string(),
            runs: p.runs,
            passed: p.passed,
            decided: p.decided,
            pass_rate: p.pass_rate,
            mean_score: p.mean_score,
            cost_usd: p.cost_usd,
        })
        .collect())
}

#[cfg(not(feature = "farm"))]
async fn eval_results(
    _ctx: &ToolCtx<'_>,
    _input: FarmEvalResultsInput,
) -> ToolResult<(EvalResultsOutput, String)> {
    Err(super::ToolError::Invalid(
        "farm evals are not available on this instance".to_owned(),
    ))
}

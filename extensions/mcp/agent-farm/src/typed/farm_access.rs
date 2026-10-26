//! The farm entitlement every farm-touching tool shares and the farm's own
//! record of a run (its `agent_farm_runs` row and the deterministic
//! assertion result it produced, if any).

use systemprompt::agent_runs::AgentRunRow;
use systemprompt_web_admin_farm::repositories::rows::{FarmRow, RunRow};
use systemprompt_web_admin_farm::repositories::runs::find_run_by_agent_run;
use systemprompt_web_admin_farm::repositories::{evals, farms};
use systemprompt_web_admin_farm::services::entitlement::may_use_farm;
use systemprompt_web_admin_farm::services::runner::ORIGIN_KIND;
use systemprompt_web_admin_farm::types::{EvalResultId, FarmId};

use crate::typed::outputs::{AssertionOutcome, FarmRunExtras, RunExtras};
use crate::typed::{ToolCtx, ToolError, ToolResult};

pub(crate) async fn entitled_farm(ctx: &ToolCtx<'_>, farm: &str) -> ToolResult<FarmRow> {
    let id = FarmId::new(crate::typed::non_empty("farm", farm)?);
    let row = farms::find_farm(ctx.pool()?, &id)
        .await?
        .ok_or_else(|| ToolError::NotFound(format!("farm {id} does not exist")))?;
    let allowed = may_use_farm(ctx.pool()?, ctx.services()?, ctx.viewer.context()?, &row).await?;
    if !allowed {
        return Err(ToolError::Forbidden(format!(
            "farm {} requires the workspace {}",
            row.id, row.marketplace_id
        )));
    }
    Ok(row)
}

pub(crate) async fn farm_run_of(
    ctx: &ToolCtx<'_>,
    run: &AgentRunRow,
) -> ToolResult<Option<RunRow>> {
    if run.origin_kind != ORIGIN_KIND {
        return Ok(None);
    }
    Ok(find_run_by_agent_run(ctx.pool()?, &run.id).await?)
}

pub(crate) async fn farm_extras(
    ctx: &ToolCtx<'_>,
    run: &AgentRunRow,
) -> ToolResult<Option<RunExtras>> {
    let Some(farm_run) = farm_run_of(ctx, run).await? else {
        return Ok(None);
    };
    let result = match farm_run.eval_result_id.as_deref() {
        Some(id) => evals::find_eval_result(ctx.pool()?, &EvalResultId::new(id)).await?,
        None => None,
    };
    let (eval_run_id, eval_case_id, eval_status, assertions) = match result {
        Some(result) => (
            Some(result.eval_run_id.as_str().to_owned()),
            Some(result.case_id),
            Some(result.status),
            serde_json::from_value::<Vec<AssertionOutcome>>(result.assertions)?,
        ),
        None => (None, None, None, Vec::new()),
    };
    Ok(Some(RunExtras::Farm(Box::new(FarmRunExtras {
        farm_run_id: farm_run.id.as_str().to_owned(),
        farm: farm_run.farm_id.as_str().to_owned(),
        agent: farm_run.agent_id.as_str().to_owned(),
        triggered_via: farm_run.triggered_via,
        prompt: farm_run.prompt,
        model: farm_run.model,
        eval_result_id: farm_run.eval_result_id,
        eval_run_id,
        eval_case_id,
        eval_status,
        assertions,
    }))))
}

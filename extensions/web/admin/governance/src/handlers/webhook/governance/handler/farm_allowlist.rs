//! The `farm_tool_allowlist` stage: a farm run's governance hook refuses any
//! `mcp__` tool its agent did not declare.
//!
//! It runs ahead of the configured chain and is enforced whatever the chain's
//! mode, because it is the only line that can name an undeclared MCP tool
//! (see the farm runner's `tool_policy`). A session that belongs to no farm
//! run is untouched.

use sqlx::PgPool;
use systemprompt::governance::authz::{Decision, DenyReason};
use systemprompt::governance::policy::{
    ChainEntryOutcome, ChainEntryResult, Evaluation, GovernedTarget,
};
use systemprompt::identifiers::{PolicyId, SessionId};
use systemprompt_web_admin_core::util::tool_allowlist::{
    FARM_TOOL_ALLOWLIST_POLICY, MCP_TOOL_PREFIX, undeclared_mcp_tool,
};

use crate::repositories::governance::farm_allowlist::find_farm_run_allowlist;

pub(super) async fn allowlist_violation(
    pool: &PgPool,
    session_id: Option<&SessionId>,
    target: &GovernedTarget,
) -> Result<Option<String>, &'static str> {
    let (Some(session_id), GovernedTarget::Tool { tool }) = (session_id, target) else {
        return Ok(None);
    };
    if !tool.as_str().starts_with(MCP_TOOL_PREFIX) {
        return Ok(None);
    }
    let allowlist = find_farm_run_allowlist(pool, session_id)
        .await
        .map_err(|error| {
            tracing::error!(%error, "farm tool allowlist unavailable; denying execution");
            "Farm tool allowlist unavailable"
        })?;
    Ok(allowlist
        .filter(|tools| undeclared_mcp_tool(tool.as_str(), tools))
        .map(|_| format!("{tool} is not in this farm agent's declared tools")))
}

pub(super) fn enforce(evaluation: &mut Evaluation, detail: String) {
    evaluation.chain.insert(
        0,
        ChainEntryOutcome {
            policy_id: PolicyId::new(FARM_TOOL_ALLOWLIST_POLICY),
            result: ChainEntryResult::Fail,
            detail: detail.clone(),
            duration_ms: 0.0,
        },
    );
    evaluation.decision = Decision::Deny {
        reason: DenyReason::PolicyViolation {
            policy: FARM_TOOL_ALLOWLIST_POLICY.to_owned(),
            detail: detail.into(),
        },
    };
}

//! The server-side audit row every Odoo tool call lands, allowed or refused.
//!
//! Client-side governance hooks cannot be relied on here: a person's client
//! may not run them and the farm runner drops kit hooks. So this server writes
//! its own `governance_decisions` row per call through core's single writer,
//! attributed to the caller's user id, with the farm run id, the tool, the
//! Odoo model and the record ids in `evaluated_rules`. A refusal by the
//! destructive-role check carries that policy's name.

use sqlx::PgPool;
use systemprompt::identifiers::Actor;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::security::authz::{
    DecisionTag, GovernanceDecisionRecord, insert_governance_decision,
};

use crate::caller::Caller;

pub const AUDIT_POLICY: &str = "odoo_tool_audit";

const RECORD_ID_KEYS: [&str; 5] = ["id", "partner_id", "res_id", "lead_id", "channel_id"];

#[derive(Debug, Clone, Copy)]
pub struct Outcome<'a> {
    pub tag: DecisionTag,
    pub policy: &'a str,
    pub reason: &'a str,
}

#[must_use]
pub fn model_for_tool(tool: &str) -> &'static str {
    const MODELS: [(&str, &str); 11] = [
        ("crm_", "crm.lead"),
        ("partner_", "res.partner"),
        ("gdpr_", "res.partner"),
        ("note_", "mail.message"),
        ("attachment_", "ir.attachment"),
        ("activity_", "mail.activity"),
        ("calendar_", "calendar.event"),
        ("task_", "project.task"),
        ("channel_", "discuss.channel"),
        ("sale_order_", "sale.order"),
        ("invoice_", "account.move"),
    ];
    MODELS
        .iter()
        .find(|(prefix, _)| tool.starts_with(prefix))
        .map_or("", |(_, model)| *model)
}

// JSON: MCP tool arguments are a JSON object at the protocol boundary.
#[must_use]
pub fn record_ids(arguments: Option<&serde_json::Map<String, serde_json::Value>>) -> Vec<i64> {
    let Some(arguments) = arguments else {
        return Vec::new();
    };
    RECORD_ID_KEYS
        .iter()
        .filter_map(|key| arguments.get(*key).and_then(serde_json::Value::as_i64))
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct CallAudit<'a> {
    pub caller: &'a Caller,
    pub context: &'a RequestContext,
    pub tool: &'a str,
    // JSON: MCP tool arguments are a JSON object at the protocol boundary.
    pub arguments: Option<&'a serde_json::Map<String, serde_json::Value>>,
}

pub async fn record(pool: &PgPool, call: CallAudit<'_>, outcome: Outcome<'_>) {
    let id = uuid::Uuid::new_v4().to_string();
    // JSON: variable-shape governance audit evaluated_rules payload
    let evaluated = serde_json::json!({
        "server": crate::tools::SERVER_NAME,
        "tool": call.tool,
        "model": model_for_tool(call.tool),
        "record_ids": record_ids(call.arguments),
        "farm_run_id": call.caller.run_id,
        "roles": call.caller.roles,
    });
    let actor = Actor::user(call.caller.user_id.clone());
    let tool_name = format!("mcp__{}__{}", crate::tools::SERVER_NAME, call.tool);
    let row = GovernanceDecisionRecord {
        id: &id,
        actor: &actor,
        session_id: Some(call.context.session_id()),
        tool_name: &tool_name,
        agent_id: None,
        agent_scope: None,
        decision: outcome.tag,
        policy: outcome.policy,
        reason: outcome.reason,
        evaluated_rules: &evaluated,
        plugin_id: None,
        act_chain: &[],
        context_id: call.context.context_id(),
        task_id: call.context.task_id(),
        trace_id: Some(call.context.trace_id()),
        client_id: None,
        tool_use_id: None,
    };
    if let Err(e) = insert_governance_decision(pool, &row).await {
        tracing::error!(error = %e, tool = call.tool, "Failed to record the Odoo tool audit row");
    }
}

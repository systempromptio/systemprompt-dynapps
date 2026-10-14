//! The server-side gate every call passes, for people and service accounts
//! alike: the destructive-role check, the audit row, and the chatter
//! attribution note left on the record a write touched.

use std::collections::HashSet;
use std::sync::OnceLock;

use rmcp::model::{CallToolResult, ContentBlock};
use sqlx::PgPool;
use systemprompt::security::authz::DecisionTag;

use super::call::OdooCall;
use crate::audit::{AUDIT_POLICY, CallAudit, Outcome, model_for_tool, record, record_ids};
use crate::caller::{DESTRUCTIVE_POLICY, DESTRUCTIVE_ROLE, needs_destructive_authority};
use crate::tools::{
    TOOL_CHANNEL_POST, TOOL_LEAD_DELETE, TOOL_NOTE_ADD, TOOL_PARTNER_DELETE, list_tools,
};

// Why: these post their own chatter (the footer rides on the post) or remove
// the record a note would hang off.
const SELF_ATTRIBUTING: [&str; 4] = [
    TOOL_NOTE_ADD,
    TOOL_CHANNEL_POST,
    TOOL_LEAD_DELETE,
    TOOL_PARTNER_DELETE,
];

// Why: Odoo's mail models have no chatter of their own; a note about them
// would be posted nowhere a reader looks.
const UNANNOTATED_MODELS: [&str; 4] = ["mail.activity", "mail.message", "ir.attachment", ""];

fn write_tools() -> &'static HashSet<String> {
    static WRITES: OnceLock<HashSet<String>> = OnceLock::new();
    WRITES.get_or_init(|| {
        list_tools()
            .into_iter()
            .filter(|tool| tool.annotations.as_ref().and_then(|a| a.read_only_hint) != Some(true))
            .map(|tool| tool.name.to_string())
            .collect()
    })
}

#[must_use]
pub fn is_write_tool(tool: &str) -> bool {
    write_tools().contains(tool)
}

#[must_use]
pub fn refusal_reason(tool: &str) -> String {
    format!(
        "Refused: {tool} requires an administrator or the `{DESTRUCTIVE_ROLE}` role, and the \
         caller holds neither. An administrator can grant `{DESTRUCTIVE_ROLE}` for the run and \
         revoke it afterwards; nothing in Odoo was read or changed."
    )
}

#[must_use]
pub fn policy_for(tool: &str) -> &'static str {
    if needs_destructive_authority(tool) {
        DESTRUCTIVE_POLICY
    } else {
        AUDIT_POLICY
    }
}

pub(super) async fn authorise(pool: &PgPool, audit: CallAudit<'_>) -> Result<(), CallToolResult> {
    if !needs_destructive_authority(audit.tool) || audit.caller.may_act_destructively() {
        return Ok(());
    }
    let reason = refusal_reason(audit.tool);
    let outcome = Outcome {
        tag: DecisionTag::Deny,
        policy: DESTRUCTIVE_POLICY,
        reason: &reason,
    };
    record(pool, audit, outcome).await;
    Err(CallToolResult::error(vec![ContentBlock::text(reason)]))
}

pub(super) async fn refuse(pool: &PgPool, audit: CallAudit<'_>, reason: &str) -> CallToolResult {
    let outcome = Outcome {
        tag: DecisionTag::Deny,
        policy: policy_for(audit.tool),
        reason,
    };
    record(pool, audit, outcome).await;
    CallToolResult::error(vec![ContentBlock::text(reason.to_owned())])
}

pub(super) async fn settle(pool: &PgPool, audit: CallAudit<'_>, succeeded: bool) {
    let reason = if succeeded {
        "executed against Odoo"
    } else {
        "Odoo call failed"
    };
    let outcome = Outcome {
        tag: DecisionTag::Allow,
        policy: policy_for(audit.tool),
        reason,
    };
    record(pool, audit, outcome).await;
}

// JSON: MCP tool arguments are a JSON object at the protocol boundary.
fn target(
    tool: &str,
    arguments: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Option<(String, i64)> {
    let named = arguments
        .and_then(|a| a.get("model").or_else(|| a.get("res_model")))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    let model = named.unwrap_or_else(|| model_for_tool(tool).to_owned());
    if UNANNOTATED_MODELS.contains(&model.as_str()) {
        return None;
    }
    record_ids(arguments).first().map(|id| (model, *id))
}

fn field_label(field: &str) -> String {
    match field {
        "tag_ids" | "category_id" => "tags".to_owned(),
        "user_id" => "salesperson".to_owned(),
        "user_ids" => "assignees".to_owned(),
        other => other.trim_end_matches("_id").replace('_', " "),
    }
}

// JSON: MCP tool arguments are a JSON object at the protocol boundary.
#[must_use]
pub fn action_phrase(
    tool: &str,
    arguments: Option<&serde_json::Map<String, serde_json::Value>>,
) -> String {
    let mut changed: Vec<String> = arguments
        .and_then(|a| a.get("fields"))
        .and_then(serde_json::Value::as_object)
        .map(|fields| fields.keys().map(|k| field_label(k)).collect())
        .unwrap_or_default();
    for key in ["stage", "active", "name", "date_deadline"] {
        if arguments.is_some_and(|a| a.get(key).is_some_and(|v| !v.is_null())) {
            changed.push(field_label(key));
        }
    }
    match tool {
        "crm_lead_mark_won" => "Marked won".to_owned(),
        "crm_lead_mark_lost" => "Marked lost".to_owned(),
        "crm_lead_convert_to_opportunity" => "Converted to an opportunity".to_owned(),
        "partner_anonymise" => "Anonymised personal data".to_owned(),
        "activity_create" => "Scheduled an activity".to_owned(),
        _ if tool.ends_with("_update") && !changed.is_empty() => {
            changed.dedup();
            format!("Updated {}", changed.join(", "))
        },
        _ => format!("Ran {tool}"),
    }
}

// JSON: MCP tool arguments are a JSON object at the protocol boundary.
pub(super) async fn post_attribution(
    call: &OdooCall,
    tool: &str,
    arguments: Option<&serde_json::Map<String, serde_json::Value>>,
) {
    if !is_write_tool(tool) || SELF_ATTRIBUTING.contains(&tool) {
        return;
    }
    let Some((model, id)) = target(tool, arguments) else {
        return;
    };
    let mut creds = call.creds.clone();
    let footer = creds.footer.take().unwrap_or_default();
    let body = format!("{} {footer}", action_phrase(tool, arguments));
    if let Err(e) = call.client.message_post(&creds, &model, id, &body).await {
        tracing::warn!(error = %e, tool, model, id, "Could not post the Dynapps attribution note");
    }
}

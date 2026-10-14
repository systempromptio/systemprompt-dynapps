//! The guarded tools: `crm_lead_delete`, `partner_delete`, and the erasure
//! pair in [`erasure`].
//!
//! Each delete is a real Odoo `unlink`, not an archive — archiving already
//! exists through the update tools, and a delete-named tool that only archived
//! would leave the governance blocklist protecting nothing. Before unlinking,
//! the handler reads the record (so a missing id fails cleanly and its name
//! survives into the result), requires a `reason`, and applies demo scope
//! through [`crate::guard`]. Odoo's own ACL is the last gate.

pub mod erasure;
pub mod erasure_shape;

use rmcp::ErrorData as McpError;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext;

use super::call::OdooCall;
use crate::format::{field, text_artifact};
use crate::guard::{
    Target, check_demo_scope, demo_scope_enabled, irreversible_notice, require_reason,
};
use crate::tools::inputs::{LeadDeleteInput, PartnerDeleteInput};
use crate::tools::{TOOL_LEAD_DELETE, TOOL_PARTNER_DELETE};

/// How one model links a record to its tags, for the demo-scope check.
#[derive(Debug, Clone, Copy)]
pub struct Tagged {
    pub model: &'static str,
    pub label: &'static str,
    pub tag_field: &'static str,
    pub tag_model: &'static str,
    pub tag_kind: &'static str,
}

pub const LEAD: Tagged = Tagged {
    model: "crm.lead",
    label: "lead",
    tag_field: "tag_ids",
    tag_model: "crm.tag",
    tag_kind: "tag",
};

pub const PARTNER: Tagged = Tagged {
    model: "res.partner",
    label: "partner",
    tag_field: "category_id",
    tag_model: "res.partner.category",
    tag_kind: "category",
};

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
fn ids_of(record: &serde_json::Value, key: &str) -> Vec<i64> {
    record
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|ids| ids.iter().filter_map(serde_json::Value::as_i64).collect())
        .unwrap_or_default()
}

pub async fn read_tagged(
    call: &OdooCall,
    kind: Tagged,
    id: i64,
    fields: &[&str],
) -> Result<(serde_json::Value, Vec<String>), McpError> {
    let mut wanted: Vec<&str> = vec!["id", "name", kind.tag_field];
    wanted.extend_from_slice(fields);
    let mut records = call
        .client
        .read(&call.creds, kind.model, &[id], &wanted)
        .await?;
    let Some(record) = records.pop() else {
        return Err(McpError::invalid_params(
            format!(
                "No {} with id {id} is visible to this Odoo account, so there is nothing to change.",
                kind.label
            ),
            None,
        ));
    };
    let tag_ids = ids_of(&record, kind.tag_field);
    let tags = if tag_ids.is_empty() {
        Vec::new()
    } else {
        call.client
            .read(&call.creds, kind.tag_model, &tag_ids, &["id", "name"])
            .await?
            .iter()
            .filter_map(|tag| field(tag, "name"))
            .collect()
    };
    Ok((record, tags))
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
pub async fn guard_record(
    call: &OdooCall,
    kind: Tagged,
    id: i64,
    fields: &[&str],
) -> Result<serde_json::Value, McpError> {
    let (record, tags) = read_tagged(call, kind, id, fields).await?;
    let target = Target {
        label: kind.label,
        id,
        tag_kind: kind.tag_kind,
    };
    check_demo_scope(demo_scope_enabled(), target, &tags)?;
    Ok(record)
}

pub async fn delete_record(
    call: &OdooCall,
    kind: Tagged,
    id: i64,
    reason: String,
) -> Result<(CliArtifact, String), McpError> {
    let record = guard_record(call, kind, id, &[]).await?;
    let name = field(&record, "name").unwrap_or_else(|| "untitled".to_owned());

    let acknowledged = call.client.unlink(&call.creds, kind.model, &[id]).await?;
    if !acknowledged {
        return Err(McpError::internal_error(
            format!("Odoo did not acknowledge deleting {} {id}.", kind.label),
            None,
        ));
    }

    let notice = irreversible_notice(
        &format!("{} {id} (\"{name}\") was unlinked from Odoo", kind.label),
        &reason,
    );
    let summary = format!(
        "Deleted Odoo {} {id} (\"{name}\") as {} — irreversible",
        kind.label, call.creds.login
    );
    let body = format!(
        "Permanently deleted {} **[{id}] {name}** from Odoo as `{}`. The record, its chatter \
         and its activities cannot be recovered.\n\n{notice}",
        kind.label, call.creds.login
    );
    Ok((text_artifact("Record Deleted", &body), summary))
}

#[derive(Debug)]
pub struct LeadDeleteHandler {
    pub call: OdooCall,
}

impl McpToolHandler for LeadDeleteHandler {
    type Input = LeadDeleteInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_LEAD_DELETE
    }

    fn description(&self) -> &'static str {
        "Permanently delete a lead from Odoo CRM."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let call = self.call.clone();
        async move {
            let reason = require_reason(TOOL_LEAD_DELETE, &input.reason)?;
            delete_record(&call, LEAD, input.id, reason).await
        }
    }
}

#[derive(Debug)]
pub struct PartnerDeleteHandler {
    pub call: OdooCall,
}

impl McpToolHandler for PartnerDeleteHandler {
    type Input = PartnerDeleteInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_PARTNER_DELETE
    }

    fn description(&self) -> &'static str {
        "Permanently delete a contact from Odoo."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let call = self.call.clone();
        async move {
            let reason = require_reason(TOOL_PARTNER_DELETE, &input.reason)?;
            delete_record(&call, PARTNER, input.id, reason).await
        }
    }
}

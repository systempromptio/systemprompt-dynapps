//! The GDPR erasure pair: `gdpr_erase_inventory` (read-only) and
//! `partner_anonymise`.
//!
//! The inventory lists everything linked to a contact — leads, chatter count,
//! invoices with their state, attachments — so an erasure plan is written
//! from what Odoo holds rather than from what an agent remembers to look for.
//! Anonymisation overwrites the contact's personal fields and the contact
//! fields of its leads in place; it refuses while the contact has a posted
//! invoice, because posted accounting entries are under legal retention and
//! must keep the name they were issued to.

use rmcp::ErrorData as McpError;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext;

use super::erasure_shape::{
    INVENTORY_LIMIT, Linked, anonymised_lead_values, anonymised_partner_values, inventory_markdown,
    linked_leads_domain,
};
use super::{PARTNER, guard_record, read_tagged};
use crate::client::SearchOptions;
use crate::error::OdooError;
use crate::format::{field_or_dash, text_artifact};
use crate::guard::{irreversible_notice, require_reason};
use crate::server::call::OdooCall;
use crate::tools::inputs::{GdprEraseInventoryInput, PartnerAnonymiseInput};
use crate::tools::{TOOL_GDPR_ERASE_INVENTORY, TOOL_PARTNER_ANONYMISE};

fn options(fields: &[&str]) -> SearchOptions {
    SearchOptions {
        fields: fields.iter().map(|f| (*f).to_owned()).collect(),
        limit: INVENTORY_LIMIT,
        order: None,
    }
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
// Why: a deployment without Invoicing has no `account.move`; that contact has
// no invoices, which is an answer rather than a failure.
fn absent_app_is_empty(
    result: Result<Vec<serde_json::Value>, OdooError>,
) -> Result<Vec<serde_json::Value>, OdooError> {
    match result {
        Err(OdooError::AppMissing(_)) => Ok(Vec::new()),
        other => other,
    }
}

async fn linked_records(call: &OdooCall, partner_id: i64) -> Result<Linked, OdooError> {
    let client = &call.client;
    let creds = &call.creds;
    let leads = client
        .search_read(
            creds,
            "crm.lead",
            linked_leads_domain(partner_id),
            &options(&["id", "name", "active", "email_from"]),
        )
        .await?;
    let moves = absent_app_is_empty(
        client
            .search_read(
                creds,
                "account.move",
                serde_json::json!([["partner_id", "=", partner_id]]),
                &options(&["id", "name", "state", "move_type"]),
            )
            .await,
    )?;
    let attachments = client
        .search_read(
            creds,
            "ir.attachment",
            serde_json::json!([
                ["res_model", "=", "res.partner"],
                ["res_id", "=", partner_id]
            ]),
            &options(&["id", "name", "mimetype"]),
        )
        .await?;
    let messages = client
        .search_read(
            creds,
            "mail.message",
            serde_json::json!([["model", "=", "res.partner"], ["res_id", "=", partner_id]]),
            &options(&["id"]),
        )
        .await?
        .len();
    Ok(Linked {
        leads,
        moves,
        attachments,
        messages,
    })
}

pub async fn anonymise(
    call: &OdooCall,
    id: i64,
    reason: String,
) -> Result<(CliArtifact, String), McpError> {
    let record = guard_record(call, PARTNER, id, &[]).await?;
    let name = field_or_dash(&record, "name");
    let linked = linked_records(call, id).await?;

    let posted = linked.posted_moves();
    if !posted.is_empty() {
        let listed: Vec<String> = posted
            .iter()
            .map(|m| format!("[{}] {}", field_or_dash(m, "id"), field_or_dash(m, "name")))
            .collect();
        return Err(McpError::invalid_request(
            format!(
                "Refused: partner {id} has {} posted invoice(s) or entries under legal \
                 retention ({}). Nothing was changed. Leave them intact and record the \
                 retention in the erasure report.",
                posted.len(),
                listed.join(", ")
            ),
            None,
        ));
    }

    call.client
        .write(
            &call.creds,
            "res.partner",
            id,
            anonymised_partner_values(id),
        )
        .await?;
    for lead in &linked.leads {
        // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
        if let Some(lead_id) = lead.get("id").and_then(serde_json::Value::as_i64) {
            call.client
                .write(&call.creds, "crm.lead", lead_id, anonymised_lead_values(id))
                .await?;
        }
    }

    let notice = irreversible_notice(
        &format!(
            "partner {id} and {} linked lead(s) had their personal fields overwritten",
            linked.leads.len()
        ),
        &reason,
    );
    let summary = format!(
        "Anonymised partner {id} and {} lead(s) as {} — irreversible",
        linked.leads.len(),
        call.creds.login
    );
    let body = format!(
        "Partner **[{id}] {name}** is now `Anonymised {id}`; its email, phone, address, \
         VAT, website and notes are cleared, and the contact fields of {} linked lead(s) \
         are overwritten. Chatter ({} message(s)) and attachments ({}) are untouched.\
         \n\n{notice}",
        linked.leads.len(),
        linked.messages,
        linked.attachments.len()
    );
    Ok((text_artifact("Contact Anonymised", &body), summary))
}

#[derive(Debug)]
pub struct GdprEraseInventoryHandler {
    pub call: OdooCall,
}

impl McpToolHandler for GdprEraseInventoryHandler {
    type Input = GdprEraseInventoryInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_GDPR_ERASE_INVENTORY
    }

    fn description(&self) -> &'static str {
        "List every Odoo record linked to a contact, for an erasure request."
    }

    fn read_only(&self) -> bool {
        true
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let call = self.call.clone();
        async move {
            let (record, _tags) = read_tagged(&call, PARTNER, input.partner_id, &[]).await?;
            let name = field_or_dash(&record, "name");
            let linked = linked_records(&call, input.partner_id).await?;
            let summary = format!(
                "Partner {}: {} lead(s), {} invoice(s) ({} posted), {} attachment(s), {} message(s)",
                input.partner_id,
                linked.leads.len(),
                linked.moves.len(),
                linked.posted_moves().len(),
                linked.attachments.len(),
                linked.messages
            );
            let body = inventory_markdown(input.partner_id, &name, &linked);
            Ok((text_artifact("Erasure Inventory", &body), summary))
        }
    }
}

#[derive(Debug)]
pub struct PartnerAnonymiseHandler {
    pub call: OdooCall,
}

impl McpToolHandler for PartnerAnonymiseHandler {
    type Input = PartnerAnonymiseInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        TOOL_PARTNER_ANONYMISE
    }

    fn description(&self) -> &'static str {
        "Overwrite a contact's personal data, and its leads' contact fields, in place."
    }

    fn handle(
        &self,
        input: Self::Input,
        _ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> impl Future<Output = Result<(Self::Output, String), McpError>> + Send {
        let call = self.call.clone();
        async move {
            let reason = require_reason(TOOL_PARTNER_ANONYMISE, &input.reason)?;
            anonymise(&call, input.id, reason).await
        }
    }
}

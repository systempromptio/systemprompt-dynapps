//! The guarded tools: the two deletes and the GDPR erasure pair.
//!
//! Every changing tool here is destructive by annotation, takes a required
//! `reason`, and — while `ODOO_DEMO_SCOPE` is on — only touches records tagged
//! `demo-*`. The descriptions say so, because an agent that knows a call will
//! be refused can plan around it instead of discovering it.

use rmcp::model::Tool;

use super::super::inputs::{
    GdprEraseInventoryInput, LeadDeleteInput, PartnerAnonymiseInput, PartnerDeleteInput,
};
use super::super::{
    Effect, TOOL_GDPR_ERASE_INVENTORY, TOOL_LEAD_DELETE, TOOL_PARTNER_ANONYMISE,
    TOOL_PARTNER_DELETE, ToolDef, create_tool,
};

pub fn guarded_tools() -> Vec<Tool> {
    vec![
        create_tool(&ToolDef {
            name: TOOL_LEAD_DELETE,
            title: "Delete CRM Lead",
            description: "Permanently delete a lead from Odoo CRM by id. This is `unlink`, not \
                          archive — the record, its chatter and its activities are gone and \
                          cannot be recovered. Use crm_lead_update with `{\"active\": false}` \
                          to archive instead. Requires a `reason`; while demo scope is on, only \
                          leads tagged `demo-*` can be deleted.",
            input_schema: schemars::schema_for!(LeadDeleteInput).to_value(),
            effect: Effect::Destructive,
        }),
        create_tool(&ToolDef {
            name: TOOL_PARTNER_DELETE,
            title: "Delete Contact",
            description: "Permanently delete a contact (`res.partner`) by id — `unlink`, not \
                          archive. Odoo refuses it while invoices or orders still reference the \
                          contact. Requires a `reason`; while demo scope is on, only contacts \
                          in a `demo-*` category can be deleted.",
            input_schema: schemars::schema_for!(PartnerDeleteInput).to_value(),
            effect: Effect::Destructive,
        }),
        create_tool(&ToolDef {
            name: TOOL_PARTNER_ANONYMISE,
            title: "Anonymise Contact",
            description: "Fulfil an erasure request: overwrite a contact's name with \
                          `Anonymised <id>`, clear its email, phone, address, VAT, website and \
                          notes, and clear the contact fields of every linked lead. Refused \
                          while the contact has a posted invoice (legal retention) — the refusal \
                          lists them. Irreversible; requires a `reason`; demo-scoped like the \
                          deletes. Call gdpr_erase_inventory first.",
            input_schema: schemars::schema_for!(PartnerAnonymiseInput).to_value(),
            effect: Effect::Destructive,
        }),
        create_tool(&ToolDef {
            name: TOOL_GDPR_ERASE_INVENTORY,
            title: "Erasure Inventory",
            description: "Read-only: list every record Odoo links to a contact — leads \
                          (archived included), invoices and entries with their state, \
                          attachments, and the chatter message count. Use it to plan an \
                          erasure request before partner_anonymise.",
            input_schema: schemars::schema_for!(GdprEraseInventoryInput).to_value(),
            effect: Effect::ReadOnly,
        }),
    ]
}

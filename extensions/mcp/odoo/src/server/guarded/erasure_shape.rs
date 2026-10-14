//! The pure half of the erasure tools: which fields are personal data, the
//! values that overwrite them, and how the linked-record inventory reads.
//!
//! Kept apart from [`super::erasure`] so the Odoo calls and the rules they
//! apply can be read, and tested, separately.

use crate::format::{field, field_or_dash};

pub const INVENTORY_LIMIT: u32 = 200;

pub const PARTNER_PII: [&str; 10] = [
    "email", "phone", "mobile", "street", "street2", "city", "zip", "vat", "website", "comment",
];

pub const LEAD_PII: [&str; 9] = [
    "contact_name",
    "partner_name",
    "email_from",
    "phone",
    "street",
    "street2",
    "city",
    "zip",
    "website",
];

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
// Why: Odoo hides archived records from a search unless the domain asks for
// both states, and an erasure that misses archived leads is not an erasure.
pub fn linked_leads_domain(partner_id: i64) -> serde_json::Value {
    serde_json::json!([
        ["partner_id", "=", partner_id],
        "|",
        ["active", "=", true],
        ["active", "=", false]
    ])
}

/// The records linked to one contact, as the inventory and the anonymiser
/// both need them.
#[derive(Debug, Default)]
pub struct Linked {
    pub leads: Vec<serde_json::Value>,
    pub moves: Vec<serde_json::Value>,
    pub attachments: Vec<serde_json::Value>,
    pub messages: usize,
}

impl Linked {
    #[must_use]
    pub fn posted_moves(&self) -> Vec<&serde_json::Value> {
        self.moves
            .iter()
            .filter(|m| field(m, "state").as_deref() == Some("posted"))
            .collect()
    }
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
fn bullet_list(records: &[serde_json::Value], keys: &[&str]) -> String {
    if records.is_empty() {
        return "- none".to_owned();
    }
    records
        .iter()
        .map(|r| {
            let detail: Vec<String> = keys.iter().map(|k| field_or_dash(r, k)).collect();
            format!("- [{}] {}", field_or_dash(r, "id"), detail.join(" · "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[must_use]
pub fn inventory_markdown(partner_id: i64, name: &str, linked: &Linked) -> String {
    format!(
        "Everything Odoo links to partner **[{partner_id}] {name}**:\n\n### Leads ({})\n{}\n\n\
         ### Invoices and entries ({}, {} posted)\n{}\n\n### Attachments ({})\n{}\n\n\
         ### Chatter\n- {} message(s) on the contact",
        linked.leads.len(),
        bullet_list(&linked.leads, &["name", "email_from"]),
        linked.moves.len(),
        linked.posted_moves().len(),
        bullet_list(&linked.moves, &["name", "state", "move_type"]),
        linked.attachments.len(),
        bullet_list(&linked.attachments, &["name", "mimetype"]),
        linked.messages,
    )
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
#[must_use]
pub fn anonymised_partner_values(id: i64) -> serde_json::Value {
    let mut values = serde_json::Map::new();
    values.insert(
        "name".to_owned(),
        serde_json::json!(format!("Anonymised {id}")),
    );
    for key in PARTNER_PII {
        values.insert(key.to_owned(), serde_json::Value::Bool(false));
    }
    serde_json::Value::Object(values)
}

#[must_use]
pub fn anonymised_lead_values(partner_id: i64) -> serde_json::Value {
    let mut values = serde_json::Map::new();
    for key in LEAD_PII {
        values.insert(key.to_owned(), serde_json::Value::Bool(false));
    }
    values.insert(
        "contact_name".to_owned(),
        serde_json::json!(format!("Anonymised {partner_id}")),
    );
    serde_json::Value::Object(values)
}

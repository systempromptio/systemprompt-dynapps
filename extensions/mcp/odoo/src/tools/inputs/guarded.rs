//! Inputs for the guarded tools: the deletes, the anonymisation and the
//! erasure inventory.
//!
//! Every tool here that changes Odoo takes a `reason`. It is required in the
//! schema and refused when blank, so an irreversible call always carries, in
//! the tool-call audit row, the words of whoever asked for it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LeadDeleteInput {
    pub id: i64,
    /// Why this lead is being deleted, e.g. "duplicate of lead 41". Required
    /// and recorded with the call.
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartnerDeleteInput {
    pub id: i64,
    /// Why this contact is being deleted, e.g. "duplicate of partner 12".
    /// Required and recorded with the call.
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PartnerAnonymiseInput {
    pub id: i64,
    /// The erasure request this fulfils, e.g. "GDPR art. 17 request of
    /// 2026-10-14". Required and recorded with the call.
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct GdprEraseInventoryInput {
    pub partner_id: i64,
}

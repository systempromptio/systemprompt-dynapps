//! Wire types for the bootstrap AD-group gate loader
//! (`access-control/adfs.yaml`).
//!
//! The file lists entities confined to members of named AD groups. Access is
//! not authored per grant: every row this file produces is
//! `rule_type = 'adfs_group'`, `access = allow`, `default_included = false` —
//! a grant with any other shape would not be a group gate, so the schema
//! cannot express one.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdfsGroupsDoc {
    #[serde(default)]
    pub grants: Vec<AdfsGroupGrant>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdfsGroupGrant {
    pub entity_type: String,
    pub entity_id: String,
    pub groups: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AdfsGroupsLoadReport {
    pub grants_projected: usize,
}

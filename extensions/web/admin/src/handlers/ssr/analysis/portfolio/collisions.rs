//! Unresolved names: a configured entry on disk that no metric can attach to
//! until an admin says which managed resource it is.

use super::rows::InventoryRow;
use serde::Serialize;
use std::collections::HashSet;
use systemprompt::identifiers::InventoryEntryId;
use systemprompt::marketplace::managed::ResourceSummary;

/// One unresolved name: a configured entry on disk that no metric can attach
/// to until an admin says which managed resource it is.
///
/// Two entries reach this list — a `Conflicting` entry, where a file in the
/// composed `services/` tree and an imported managed resource claim the same
/// key, and an unbound entry that has candidates to bind to. Both are resolved
/// by the same form, so both are offered in the same place rather than in a
/// control buried in the table.
#[derive(Debug, Clone, Serialize)]
pub struct Collision {
    pub entry_id: InventoryEntryId,
    pub resource_key: String,
    pub configured_key: String,
    pub conflicting: bool,
    pub candidates: Vec<ResourceSummary>,
}

// Why: one collision per inventory entry, not per row — a skill carried by
// several marketplaces repeats as a row, but it is one thing to resolve.
#[must_use]
pub fn collisions(rows: &[InventoryRow]) -> Vec<Collision> {
    let mut seen: HashSet<&InventoryEntryId> = HashSet::new();
    rows.iter()
        .filter(|row| row.usage_status == "conflicting" || !row.bind_candidates.is_empty())
        .filter(|row| seen.insert(&row.entry.entry_id))
        .map(|row| Collision {
            entry_id: row.entry.entry_id.clone(),
            resource_key: row.resource_key.clone(),
            configured_key: row
                .entry
                .configured_key
                .clone()
                .unwrap_or_else(|| row.resource_key.clone()),
            conflicting: row.usage_status == "conflicting",
            candidates: row.bind_candidates.clone(),
        })
        .collect()
}

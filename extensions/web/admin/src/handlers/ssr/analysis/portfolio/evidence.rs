//! The evidence a portfolio row is assembled from: the YAML-derived
//! marketplace/plugin index, the current audience, canonical
//! inventory, and managed resources for explicit binding actions.

use super::rows::{PortfolioResourceRow, PortfolioRowInputs};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use systemprompt::marketplace::managed::ResourceSummary;

pub(super) struct Evidence<'a> {
    pub(super) index: &'a InventoryIndex,
    pub(super) managed: Vec<ResourceSummary>,
}

impl<'a> Evidence<'a> {
    pub(super) fn gather(index: &'a InventoryIndex) -> Self {
        Self {
            managed: Vec::new(),
            index,
        }
    }
}

// Why: a skill carried by several plugins is one inventory entry but several
// audiences; each plugin gets its own row so the marketplace grouping holds.
pub(super) fn expand_rows(
    item: PortfolioEntry,
    evidence: &Evidence,
    window: (&str, &str),
) -> Vec<PortfolioResourceRow> {
    let key = item.entry.resource_key.clone();
    let plugins: Vec<(String, String)> = evidence
        .index
        .plugins_by_skill
        .get(&key)
        .map(|list| list.iter().map(|p| (p.0.clone(), p.1.clone())).collect())
        .unwrap_or_default();
    let users_with_access = None;
    if plugins.is_empty() {
        return vec![PortfolioResourceRow::build(PortfolioRowInputs {
            item,
            plugin: None,
            marketplaces: Vec::new(),
            users_with_access,
            managed: &evidence.managed,
            window,
        })];
    }
    plugins
        .iter()
        .map(|(plugin_id, plugin_name)| {
            PortfolioResourceRow::build(PortfolioRowInputs {
                item: PortfolioEntry {
                    entry: item.entry.clone(),
                    snapshot: item.snapshot.clone(),
                    installation: item.installation,
                },
                plugin: Some((plugin_id, plugin_name)),
                marketplaces: evidence
                    .index
                    .marketplaces_by_plugin
                    .get(plugin_id)
                    .cloned()
                    .unwrap_or_default(),
                users_with_access,
                managed: &evidence.managed,
                window,
            })
        })
        .collect()
}

//! The evidence a portfolio row is assembled from: the YAML-derived
//! marketplace/plugin index, the resolved audience, and the managed resources
//! that bind retained snapshots to inventory entries.

use super::rows::{InventoryRow, InventoryRowInputs};
use crate::error::AdminResult;
use crate::repositories::analysis::inventory_index::{
    InventoryIndex, MarketplaceAudience, get_marketplace_audience,
};
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use crate::repositories::analysis::publication_heads::{PublicationHead, list_publication_heads};
use crate::routes::managed_state::ManagedState;
use crate::types::Role;
use sqlx::PgPool;
use systemprompt::marketplace::managed::ResourceSummary;

pub(super) struct Evidence<'a> {
    pub(super) index: &'a InventoryIndex,
    pub(super) audience: MarketplaceAudience,
    pub(super) managed: Vec<ResourceSummary>,
    pub(super) heads: Vec<PublicationHead>,
}

impl<'a> Evidence<'a> {
    pub(super) async fn gather(
        managed: &ManagedState,
        pool: &PgPool,
        index: &'a InventoryIndex,
    ) -> AdminResult<Self> {
        let roles: Vec<String> = Role::ALL.iter().map(|r| r.as_str().to_owned()).collect();
        Ok(Self {
            audience: get_marketplace_audience(pool, &index.marketplaces, &roles).await?,
            managed: list_all_resources(managed).await?,
            heads: list_publication_heads(pool, &managed.owner).await?,
            index,
        })
    }
}

async fn list_all_resources(managed: &ManagedState) -> AdminResult<Vec<ResourceSummary>> {
    let mut all = Vec::new();
    let mut offset = 0;
    loop {
        let page = managed
            .repository
            .list_resources(&managed.owner, offset)
            .await?;
        all.extend(page.items);
        if !page.has_more {
            return Ok(all);
        }
        offset += 50;
    }
}

// Why: a skill carried by several plugins is one inventory entry but several
// audiences; each plugin gets its own row so the marketplace grouping holds.
pub(super) fn expand_rows(
    item: PortfolioEntry,
    evidence: &Evidence<'_>,
    window: (&str, &str),
) -> AdminResult<Vec<InventoryRow>> {
    let key = item.entry.resource_key.clone();
    let plugins: Vec<(String, String)> = evidence
        .index
        .plugins_by_skill
        .get(&key)
        .map(|list| list.iter().map(|p| (p.0.clone(), p.1.clone())).collect())
        .unwrap_or_default();
    let users_with_access = evidence.audience.users_reaching_skill(evidence.index, &key);
    if plugins.is_empty() {
        return Ok(vec![InventoryRow::build(InventoryRowInputs {
            item,
            plugin: None,
            marketplaces: Vec::new(),
            users_with_access,
            managed: &evidence.managed,
            heads: &evidence.heads,
            window,
        })?]);
    }
    plugins
        .iter()
        .map(|(plugin_id, plugin_name)| {
            InventoryRow::build(InventoryRowInputs {
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
                heads: &evidence.heads,
                window,
            })
        })
        .collect()
}

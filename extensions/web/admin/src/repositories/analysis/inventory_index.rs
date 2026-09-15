//! Current catalog placement only. Audience and historical group usage require
//! independent evidence.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Serialize;
use systemprompt::identifiers::{MarketplaceId, UserId};

use crate::repositories::marketplace::manifests::{
    MarketplaceConfigSummary, list_marketplace_configs,
};
use crate::repositories::marketplace::plugin_maps::{EntityPluginMap, build_entity_plugin_maps};
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Default)]
pub struct InventoryIndex {
    pub marketplaces: Vec<MarketplaceConfigSummary>,
    pub plugins_by_skill: EntityPluginMap,
    pub marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>>,
}

impl InventoryIndex {
    #[must_use]
    pub fn build(services_path: &Path) -> Self {
        let marketplaces = list_marketplace_configs(services_path).unwrap_or_else(|error| {
            tracing::warn!(error = %error, "marketplace manifests unavailable for analysis");
            Vec::new()
        });
        let (plugins_by_skill, _agents, _mcp) = build_entity_plugin_maps(services_path);
        let mut marketplaces_by_plugin: HashMap<String, Vec<MarketplaceRef>> = HashMap::new();
        for marketplace in &marketplaces {
            for plugin in &marketplace.plugins {
                marketplaces_by_plugin
                    .entry(plugin.clone())
                    .or_default()
                    .push(MarketplaceRef {
                        id: marketplace.id.as_str().to_owned(),
                        name: marketplace.name.clone(),
                    });
            }
        }
        Self {
            marketplaces,
            plugins_by_skill,
            marketplaces_by_plugin,
        }
    }

    #[must_use]
    pub fn plugin_ids_for_marketplace(&self, marketplace_id: &str) -> Option<Vec<String>> {
        self.marketplaces
            .iter()
            .find(|m| m.id.as_str() == marketplace_id)
            .map(|m| m.plugins.clone())
    }
}

/// The distinct users each marketplace reaches, resolved per group and per
/// role and then materialised through memberships.
#[derive(Debug, Default)]
pub struct MarketplaceAudience {
    pub users_by_marketplace: HashMap<String, HashSet<UserId>>,
}

impl MarketplaceAudience {
    #[must_use]
    pub fn users_reaching_skill(&self, index: &InventoryIndex, skill_id: &str) -> usize {
        let mut users: HashSet<&UserId> = HashSet::new();
        for plugin in index.plugins_by_skill.get(skill_id).into_iter().flatten() {
            for marketplace in index
                .marketplaces_by_plugin
                .get(&plugin.0)
                .into_iter()
                .flatten()
            {
                if let Some(set) = self.users_by_marketplace.get(&marketplace.id) {
                    users.extend(set.iter());
                }
            }
        }
        users.len()
    }

    #[must_use]
    pub fn users_reaching_marketplace(&self, marketplace_id: &str) -> usize {
        self.users_by_marketplace
            .get(marketplace_id)
            .map_or(0, HashSet::len)
    }
}

#[must_use]
pub fn marketplace_id(id: &MarketplaceId) -> String {
    id.as_str().to_owned()
}

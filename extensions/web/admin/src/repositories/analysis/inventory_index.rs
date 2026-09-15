//! Current catalog placement only. Audience and historical group usage require
//! independent evidence.

use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;

use crate::repositories::marketplace::manifests::{
    MarketplaceConfigSummary, list_marketplace_configs,
};
use crate::repositories::marketplace::plugin_maps::{EntityPluginMap, build_entity_plugin_maps};
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceRef {
    pub id: MarketplaceId,
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
                        id: marketplace.id.clone(),
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
}

//! Loads the configured marketplace catalog.

use systemprompt::manifest::services::{MarketplaceConfig, ServicesConfig};

#[must_use]
pub fn load_marketplaces(services: &ServicesConfig) -> Vec<MarketplaceConfig> {
    let mut entries: Vec<MarketplaceConfig> = services.marketplaces.values().cloned().collect();
    entries.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    entries
}

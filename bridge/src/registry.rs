//! Dynapps-specific bridge behaviour registered through core's `inventory`
//! seams. Linked by `main.rs` (`mod registry;`) so these submissions survive
//! into the final binary — an unreferenced module would be dropped by the
//! linker before its `inventory::submit!` initializers ran.
//!
//! This is the white-label counterpart to the build-time web overlay: the
//! overlay swaps GUI presentation, this adds server-side behaviour (host apps,
//! host syncs, marketplace sources) with no edits to core.

use systemprompt_bridge::gui::server_marketplace::MarketplaceItem;
use systemprompt_bridge::gui::server_marketplace::source::{
    MarketplaceCategory, MarketplaceSource, MarketplaceSourceCtx,
};
use systemprompt_bridge::register_marketplace_source;

struct DynappsArtifactsSource;

impl MarketplaceSource for DynappsArtifactsSource {
    fn category(&self) -> MarketplaceCategory {
        MarketplaceCategory::Artifacts
    }

    fn items(&self, _ctx: &MarketplaceSourceCtx<'_>) -> Vec<MarketplaceItem> {
        vec![
            MarketplaceItem::builder("dynapps-welcome", String::new())
                .name("Dynapps — Welcome")
                .summary(Some(
                    "Branded starter artifact contributed by the Dynapps bridge.".to_owned(),
                ))
                .build(),
        ]
    }
}

register_marketplace_source!(DynappsArtifactsSource, priority = 10);

// Why: core's manifest host gate cannot apply before the first sync, so Codex
// is suppressed here to keep it off every surface, onboarding included.
systemprompt_bridge::suppress_host_app!(systemprompt_bridge::integration::HostKind::CodexCli);

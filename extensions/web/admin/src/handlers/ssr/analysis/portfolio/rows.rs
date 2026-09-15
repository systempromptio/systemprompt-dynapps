//! Resource-wide retained usage and independent installation coverage.

use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use crate::repositories::analysis::snapshot_figures::{SnapshotFigures, cost};
use serde::Serialize;
use systemprompt::analytics::snapshots::FeedbackSnapshot;
use systemprompt::identifiers::PluginId;
use systemprompt::marketplace::inventory::{InstallationCoverage, InventoryEntry};
use systemprompt::marketplace::managed::ResourceSummary;
use systemprompt::models::feedback::inventory::InventoryAvailability;
#[path = "health.rs"]
mod health;
use super::row_display;
pub use health::{Health, HealthInputs, health_of};

#[derive(Debug, Clone, Serialize)]
pub struct PortfolioResourceRow {
    pub entry: InventoryEntry,
    pub resource_key: String,
    pub plugin_id: Option<PluginId>,
    pub plugin_name: Option<String>,
    pub marketplaces: Vec<MarketplaceRef>,
    pub catalog_href: Option<String>,
    pub drilldown_href: String,
    pub campaign_href: Option<String>,
    pub usage: Option<SnapshotFigures>,
    pub users_with_access: Option<usize>,
    pub cost: String,
    pub p95: String,
    pub avg_latency: String,
    pub tokens_in: String,
    pub tokens_out: String,
    pub tokens_cache: String,
    pub last_used: Option<String>,
    pub attributed_pct: Option<i64>,
    pub snapshot: Option<FeedbackSnapshot>,
    pub installation: Option<InstallationCoverage>,
    pub health: Health,
    pub health_tone: &'static str,
    pub health_reason: &'static str,
    pub usage_status: &'static str,
    pub bind_candidates: Vec<ResourceSummary>,
    pub more_bind_candidates: bool,
}

#[derive(Debug)]
pub struct PortfolioRowInputs<'a> {
    pub item: PortfolioEntry,
    pub plugin: Option<(&'a str, &'a str)>,
    pub marketplaces: Vec<MarketplaceRef>,
    pub users_with_access: Option<usize>,
    pub managed: &'a [ResourceSummary],
    pub window: (&'a str, &'a str),
}

const fn usage_status(item: &PortfolioEntry) -> &'static str {
    match (
        &item.snapshot,
        &item.entry.resource_id,
        item.entry.availability,
    ) {
        (_, _, InventoryAvailability::Conflicting) => "conflicting",
        (Some(snapshot), _, _) if snapshot.suppressed_days > 0 => "suppressed",
        (Some(_), _, _) => "measured",
        (None, Some(_), _) => "pending",
        (None, None, _) => "unbound",
    }
}

impl PortfolioResourceRow {
    #[must_use]
    pub fn build(input: PortfolioRowInputs<'_>) -> Self {
        let PortfolioRowInputs {
            mut item,
            plugin,
            marketplaces,
            users_with_access,
            managed,
            window,
        } = input;
        if item.entry.resource_id.is_none()
            || item
                .snapshot
                .as_ref()
                .is_some_and(|s| s.resource_id != item.entry.resource_id)
        {
            item.snapshot = None;
        }
        let usage_status = usage_status(&item);
        let usage = match item.snapshot.as_ref() {
            Some(snapshot) => SnapshotFigures::from_snapshot(snapshot),
            None => None,
        };
        let entry = item.entry;
        let binding = row_display::binding(&entry, managed);
        let (health, health_reason) =
            row_display::health(&entry, usage.as_ref(), users_with_access);
        let attributed_pct = row_display::attribution(item.snapshot.as_ref());
        let drilldown_href = format!(
            "/admin/analysis/skills?skill={}&start={}&end={}",
            urlencoding::encode(&plugin.map_or_else(
                || entry.resource_key.clone(),
                |p| usage_key(&PluginId::new(p.0), &entry.resource_key)
            )),
            window.0,
            window.1
        );
        let catalog_href = match entry.kind.as_str() {
            "skill" | "plugin" => Some(format!(
                "/admin/catalog/{}s/{}",
                entry.kind,
                urlencoding::encode(&entry.resource_key)
            )),
            _ => None,
        };
        Self {
            resource_key: entry.resource_key.clone(),
            plugin_id: plugin.map(|p| PluginId::new(p.0)),
            plugin_name: plugin.map(|p| p.1.to_owned()),
            catalog_href,
            drilldown_href,
            campaign_href: binding.campaign_href,
            users_with_access,
            cost: item
                .snapshot
                .as_ref()
                .map_or_else(|| "Unknown".to_owned(), cost),
            p95: row_display::p95(item.snapshot.as_ref()),
            avg_latency: "Unavailable".to_owned(),
            tokens_in: row_display::tokens(item.snapshot.as_ref(), true),
            tokens_out: row_display::tokens(item.snapshot.as_ref(), false),
            tokens_cache: "Unavailable".to_owned(),
            last_used: None,
            attributed_pct,
            health,
            health_tone: health.tone(),
            health_reason,
            usage_status,
            bind_candidates: binding.candidates,
            more_bind_candidates: binding.more,
            marketplaces,
            usage,
            snapshot: item.snapshot,
            installation: item.installation,
            entry,
        }
    }
}

#[must_use]
pub fn usage_key(plugin_id: &PluginId, resource_key: &str) -> String {
    format!("{plugin_id}:{}", resource_key.replace('_', "-"))
}

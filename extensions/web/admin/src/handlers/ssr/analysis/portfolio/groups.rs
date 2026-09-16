//! Marketplace → plugin → skill grouping with per-group rollups.
//!
//! A skill reachable through two plugins is listed under each, and a plugin
//! carried by two marketplaces under each marketplace: the table answers "what
//! does this audience get", so repetition across audiences is the point.
//! Rollups therefore count distinct skills, never row occurrences.

use super::rows::{Health, InventoryRow};
use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use systemprompt_web_shared::{MarketplaceId, PluginId};

#[derive(Debug, Default, Clone, Serialize)]
pub struct Rollup {
    pub skills: usize,
    pub measured: usize,
    pub used: usize,
    pub healthy: usize,
    pub warning: usize,
    pub failing: usize,
    pub invocations: i64,
    pub requests: i64,
    pub failed_requests: i64,
    pub cost_microdollars: i64,
    pub cost: String,
}

impl Rollup {
    fn absorb(&mut self, row: &InventoryRow) {
        self.skills += 1;
        if row.figures.measured {
            self.measured += 1;
        }
        if row.figures.invocations > 0 {
            self.used += 1;
        }
        match row.health {
            Health::Ok => self.healthy += 1,
            Health::Warn | Health::Muted => self.warning += 1,
            Health::Err => self.failing += 1,
        }
        self.invocations += row.figures.invocations;
        self.requests += row.figures.requests;
        self.failed_requests += row.figures.failed_requests;
        self.cost_microdollars += row.figures.cost_microdollars;
        self.cost = if self.cost_microdollars > 0 {
            format_cost(self.cost_microdollars)
        } else {
            "—".to_owned()
        };
    }
}

#[derive(Debug, Serialize)]
pub struct PluginGroup {
    pub plugin_id: Option<PluginId>,
    pub plugin_name: String,
    pub rows: Vec<InventoryRow>,
    pub rollup: Rollup,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceGroup {
    pub marketplace_id: MarketplaceId,
    pub marketplace_name: String,
    pub users_with_access: usize,
    pub plugins: Vec<PluginGroup>,
    pub rollup: Rollup,
}

pub const UNPLACED: &str = "unplaced";

type Named<T> = BTreeMap<(String, String), T>;
type Buckets = Named<Named<Vec<InventoryRow>>>;

#[must_use]
pub fn group_rows(
    rows: Vec<InventoryRow>,
    audience: impl Fn(&str) -> usize,
) -> Vec<MarketplaceGroup> {
    let mut by_marketplace: Buckets = BTreeMap::new();
    for row in rows {
        let plugin_key = (
            row.plugin_name
                .clone()
                .unwrap_or_else(|| "No plugin".to_owned()),
            row.plugin_id
                .as_ref()
                .map_or_else(String::new, |id| id.as_str().to_owned()),
        );
        if row.marketplaces.is_empty() {
            by_marketplace
                .entry(("Not in any marketplace".to_owned(), UNPLACED.to_owned()))
                .or_default()
                .entry(plugin_key)
                .or_default()
                .push(row);
            continue;
        }
        for marketplace in &row.marketplaces {
            by_marketplace
                .entry((marketplace.name.clone(), marketplace.id.clone()))
                .or_default()
                .entry(plugin_key.clone())
                .or_default()
                .push(row.clone());
        }
    }
    by_marketplace
        .into_iter()
        .map(|((name, id), plugins)| {
            let mut seen: HashSet<String> = HashSet::new();
            let mut rollup = Rollup::default();
            let plugins = plugins
                .into_iter()
                .map(|((plugin_name, plugin_id), rows)| {
                    let mut plugin_rollup = Rollup::default();
                    for row in &rows {
                        plugin_rollup.absorb(row);
                        if seen.insert(row.resource_key.clone()) {
                            rollup.absorb(row);
                        }
                    }
                    PluginGroup {
                        plugin_id: (!plugin_id.is_empty()).then(|| PluginId::new(plugin_id)),
                        plugin_name,
                        rows,
                        rollup: plugin_rollup,
                    }
                })
                .collect();
            MarketplaceGroup {
                users_with_access: audience(&id),
                marketplace_id: MarketplaceId::new(id),
                marketplace_name: name,
                plugins,
                rollup,
            }
        })
        .collect()
}

#[derive(Debug, Default, Serialize)]
pub struct Kpis {
    pub skills: usize,
    pub measured: usize,
    pub used: usize,
    pub unused: usize,
    pub failing: usize,
    pub invocations: i64,
    pub attributed: i64,
    pub users: i64,
    pub requests: i64,
    pub failed_requests: i64,
    pub cost: String,
    pub p95: String,
}

// Why: the organisation-wide snapshot carries the totals; the rows contribute
// only skill counts, deduplicated across marketplaces.
#[must_use]
pub fn kpis(rows: &[InventoryRow], totals: Option<&SnapshotFigures>) -> Kpis {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out = Kpis {
        cost: "—".to_owned(),
        p95: "—".to_owned(),
        ..Kpis::default()
    };
    for row in rows.iter().filter(|r| seen.insert(r.resource_key.as_str())) {
        out.skills += 1;
        if row.figures.measured {
            out.measured += 1;
        }
        if row.figures.invocations > 0 {
            out.used += 1;
        } else {
            out.unused += 1;
        }
        if row.health == Health::Err {
            out.failing += 1;
        }
    }
    if let Some(totals) = totals {
        out.invocations = totals.invocations;
        out.attributed = totals.verified_invocations;
        out.users = totals.distinct_users.unwrap_or(0);
        out.requests = totals.requests;
        out.failed_requests = totals.failed_requests;
        if totals.priced_requests > 0 {
            out.cost = format_cost(totals.cost_microdollars);
        }
        if let Some(micros) = totals.p95_micros {
            out.p95 = format!("≤ {}", format_duration_ms((micros / 1000) as i64));
        }
    }
    out
}

/// One marketplace as the chip row above the table shows it.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceTally {
    pub id: String,
    pub name: String,
    pub skills: usize,
    pub invocations: i64,
}

// Why: rows in no marketplace tally under `UNPLACED` rather than being
// dropped, so the single-marketplace view can still reach them. Each tally
// counts distinct skills and the invocations they drew in the window.
#[must_use]
pub fn tally_marketplaces(rows: &[InventoryRow]) -> Vec<MarketplaceTally> {
    let mut by_name: Named<(usize, i64, HashSet<String>)> = BTreeMap::new();
    for row in rows {
        let mut record = |name: String, id: String| {
            let slot = by_name.entry((name, id)).or_default();
            if slot.2.insert(row.resource_key.clone()) {
                slot.0 += 1;
                slot.1 += row.figures.invocations;
            }
        };
        if row.marketplaces.is_empty() {
            record("Not in any marketplace".to_owned(), UNPLACED.to_owned());
            continue;
        }
        for marketplace in &row.marketplaces {
            record(marketplace.name.clone(), marketplace.id.clone());
        }
    }
    by_name
        .into_iter()
        .map(|((name, id), (skills, invocations, _))| MarketplaceTally {
            id,
            name,
            skills,
            invocations,
        })
        .collect()
}

fn best_of<'a>(
    mut tallies: impl Iterator<Item = &'a MarketplaceTally>,
) -> Option<&'a MarketplaceTally> {
    let mut best = tallies.next()?;
    for tally in tallies {
        if (tally.invocations, tally.skills) > (best.invocations, best.skills) {
            best = tally;
        }
    }
    Some(best)
}

// Why: the landing page opens on one marketplace, and "the one people actually
// use" is the only defensible choice a machine can make. Skills in no
// marketplace are nobody's landing page — they are the pile you go looking for
// — so they win only when there is nothing else. With no attributed use
// anywhere the ranking falls through to the largest catalogue, then to the
// first by name, which is the order the chips are already in.
#[must_use]
pub fn busiest(tallies: &[MarketplaceTally]) -> Option<&MarketplaceTally> {
    best_of(tallies.iter().filter(|tally| tally.id != UNPLACED)).or_else(|| best_of(tallies.iter()))
}

// Why: `UNPLACED` is not a marketplace id — it means the row is delivered by
// none, so it cannot be matched against the row's own list.
#[must_use]
pub fn in_marketplace(row: &InventoryRow, marketplace: &str) -> bool {
    if marketplace == UNPLACED {
        return row.marketplaces.is_empty();
    }
    row.marketplaces.iter().any(|m| m.id == marketplace)
}

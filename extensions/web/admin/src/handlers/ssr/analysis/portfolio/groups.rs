//! Marketplace → plugin → skill grouping with per-group rollups.
//!
//! A skill reachable through two plugins is listed under each, and a plugin
//! carried by two marketplaces under each marketplace: the table answers "what
//! does this audience get", so repetition across audiences is the point.
//! Rollups therefore count distinct skills, never row occurrences.

use super::rows::{Health, SkillRow};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, Clone, Serialize)]
pub struct Rollup {
    pub skills: usize,
    pub used: usize,
    pub healthy: usize,
    pub warning: usize,
    pub failing: usize,
    pub invocations: Option<i64>,
    pub users: Option<i64>,
    pub linked_requests: Option<i64>,
    pub failed_requests: Option<i64>,
    pub cost_microdollars: Option<i64>,
    pub cost: String,
}

impl Rollup {
    fn absorb(&mut self, row: &SkillRow) {
        self.skills += 1;
        if row.usage.as_ref().is_some_and(|u| u.invocations > 0) {
            self.used += 1;
        }
        match row.health {
            Health::Ok => self.healthy += 1,
            Health::Warn | Health::Muted => self.warning += 1,
            Health::Err => self.failing += 1,
        }
        self.cost = "Unavailable for current catalog groups".to_owned();
    }
}

#[derive(Debug, Serialize)]
pub struct PluginGroup {
    pub plugin_id: String,
    pub plugin_name: String,
    pub rows: Vec<SkillRow>,
    pub rollup: Rollup,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceGroup {
    pub marketplace_id: String,
    pub marketplace_name: String,
    pub users_with_access: Option<usize>,
    pub plugins: Vec<PluginGroup>,
    pub rollup: Rollup,
}

pub const UNPLACED: &str = "unplaced";

type Named<T> = BTreeMap<(String, String), T>;
type Buckets = Named<Named<Vec<SkillRow>>>;

#[must_use]
pub fn group_rows(
    rows: Vec<SkillRow>,
    audience: impl Fn(&str) -> Option<usize>,
) -> Vec<MarketplaceGroup> {
    let mut by_marketplace: Buckets = BTreeMap::new();
    for row in rows {
        let plugin_key = (
            row.plugin_name
                .clone()
                .unwrap_or_else(|| "No plugin".to_owned()),
            row.plugin_id.clone().unwrap_or_default(),
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
                        if seen.insert(row.entry.entry_id.to_string()) {
                            rollup.absorb(row);
                        }
                    }
                    PluginGroup {
                        plugin_id,
                        plugin_name,
                        rows,
                        rollup: plugin_rollup,
                    }
                })
                .collect();
            MarketplaceGroup {
                users_with_access: audience(&id),
                marketplace_id: id,
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
    pub used: usize,
    pub unused: usize,
    pub failing: usize,
    pub invocations: Option<i64>,
    pub users: Option<i64>,
    pub linked_requests: Option<i64>,
    pub cost: String,
    pub avg_latency: String,
}

#[must_use]
pub fn kpis(
    rows: &[SkillRow],
    totals: Option<&systemprompt::analytics::snapshots::FeedbackSnapshot>,
) -> Kpis {
    let mut seen = HashSet::new();
    let mut out = Kpis::default();
    for row in rows
        .iter()
        .filter(|r| seen.insert(r.entry.entry_id.as_str()))
    {
        out.skills += 1;
        if let Some(usage) = &row.usage {
            if usage.invocations > 0 {
                out.used += 1;
            } else {
                out.unused += 1;
            }
        }
        if row.health == Health::Err {
            out.failing += 1;
        }
    }
    let totals = totals.filter(|s| s.suppressed_days == 0);
    out.invocations = totals.map(|s| s.metrics.invocations);
    out.users = totals.and_then(|s| s.distinct_users);
    out.linked_requests = totals.map(|s| s.metrics.requests);
    out.cost = totals.map_or_else(
        || "Unknown / unavailable for filter".to_owned(),
        crate::repositories::analysis::snapshot_figures::cost,
    );
    out.avg_latency = "Unavailable".to_owned();
    out
}

//! What the Skills page says *about* its rows: the marketplace picker, the
//! headline tiles, the unresolved names, the status line and the onboarding
//! steps.
//!
//! Every value here is derived from the collected rows and the retained
//! status; nothing in this module reads the database.

use super::collisions::{Collision, collisions};
use super::groups::{self, Kpis, MarketplaceTally};
use super::plan::{ALL_MARKETPLACES, Plan};
use super::{Collected, Retained};
use serde::Serialize;
use std::collections::HashSet;
use systemprompt_web_shared::PluginId;

// Why: one <option> in the toolbar's marketplace field. The skill count rides
// in the label because a bare marketplace name says nothing about whether it
// is worth switching to.
#[derive(Serialize)]
pub(crate) struct InventoryMarketplaceOption {
    id: String,
    name: String,
    skills: usize,
    is_selected: bool,
}

// Why: the three things that have to happen before a single figure on this
// page can exist, with what has happened so far.
#[derive(Serialize)]
pub(crate) struct Onboarding {
    marketplaces: bool,
    inventory: bool,
    snapshot: bool,
}

fn marketplace_options(
    tallies: &[MarketplaceTally],
    plan: &Plan,
    selected: &str,
) -> Vec<InventoryMarketplaceOption> {
    let mut out: Vec<InventoryMarketplaceOption> = tallies
        .iter()
        .map(|tally| InventoryMarketplaceOption {
            is_selected: !plan.all_marketplaces && tally.id == selected,
            id: tally.id.clone(),
            name: tally.name.clone(),
            skills: tally.skills,
        })
        .collect();
    out.push(InventoryMarketplaceOption {
        id: ALL_MARKETPLACES.to_owned(),
        name: "All marketplaces".to_owned(),
        skills: tallies.iter().map(|tally| tally.skills).sum(),
        is_selected: plan.all_marketplaces,
    });
    out
}

// Why: what the page says about the rows, as opposed to what it reads off
// them — the marketplace picker, the unresolved names, the tiles and the
// status line.
// Gathering them in one place keeps `load` a list of steps.
pub(super) struct Derived {
    pub(super) marketplace_options: Vec<InventoryMarketplaceOption>,
    pub(super) plugin_count: usize,
    pub(super) audience: usize,
    pub(super) collisions: Vec<Collision>,
    pub(super) unbound: usize,
    pub(super) conflicting: usize,
    pub(super) kpis: Kpis,
    pub(super) snapshot_state: &'static str,
    pub(super) onboarding: Option<Onboarding>,
    pub(super) synced_at: Option<String>,
    pub(super) selected_marketplace_name: String,
}

// Why: the marketplace the view settled on, the tally it was chosen from and
// the audience it reaches are three values that are only ever read together,
// and carrying them apart pushed `Derived::build` past the argument limit.
#[derive(Clone, Copy)]
pub(super) struct SelectedMarketplace<'a> {
    pub(super) tallies: &'a [MarketplaceTally],
    pub(super) id: &'a str,
    pub(super) audience: usize,
}

impl Derived {
    pub(super) fn build(
        collected: &Collected,
        plan: &Plan,
        marketplace: SelectedMarketplace<'_>,
        retained: &Retained,
    ) -> Self {
        let SelectedMarketplace {
            tallies,
            id: selected,
            audience,
        } = marketplace;
        let count = |status: &'static str| {
            collected
                .rows
                .iter()
                .filter(|row| row.usage_status == status)
                .count()
        };
        let kpis = groups::kpis(&collected.rows, retained.figures.as_ref());
        let plugins: HashSet<&str> = collected
            .rows
            .iter()
            .filter_map(|row| row.plugin_id.as_ref().map(PluginId::as_str))
            .collect();
        Self {
            marketplace_options: marketplace_options(tallies, plan, selected),
            plugin_count: plugins.len(),
            audience,
            collisions: collisions(&collected.rows),
            unbound: count("unbound"),
            conflicting: count("conflicting"),
            snapshot_state: if collected.pending_jobs > 0 || retained.health.pending_changes > 0 {
                "pending"
            } else {
                "current"
            },
            onboarding: (kpis.measured == 0).then_some(Onboarding {
                marketplaces: !tallies.is_empty(),
                inventory: retained.inventory.entries > 0,
                snapshot: retained.health.generation > 0,
            }),
            // Why: "synced" is the inventory pass, not the snapshot — the two
            // run on different clocks and the meta line names the former.
            synced_at: retained.inventory.observed_at.map(|at| at.to_rfc3339()),
            selected_marketplace_name: tallies
                .iter()
                .find(|tally| tally.id == selected)
                .map_or_else(String::new, |tally| tally.name.clone()),
            kpis,
        }
    }
}

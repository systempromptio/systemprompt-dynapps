//! What the Skills page says *about* its rows: the marketplace chip row, the
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

// Why: a chip is the tally plus the link that pins it; the link is built
// here so the row and the JSON payload can never disagree about it.
#[derive(Serialize)]
pub(crate) struct MarketplaceChip {
    id: String,
    name: String,
    skills: usize,
    href: String,
    is_active: bool,
}

// Why: the three things that have to happen before a single figure on this
// page can exist, with what has happened so far.
#[derive(Serialize)]
pub(crate) struct Onboarding {
    marketplaces: bool,
    inventory: bool,
    snapshot: bool,
}

fn chips(tallies: &[MarketplaceTally], plan: &Plan, selected: &str) -> Vec<MarketplaceChip> {
    let query = |marketplace: &str| {
        let mut href = format!("/admin/analysis/resources?days={}", plan.range.window);
        if !plan.search.is_empty() {
            href.push_str("&search=");
            href.push_str(&urlencoding::encode(&plan.search));
        }
        href.push_str("&marketplace=");
        href.push_str(&urlencoding::encode(marketplace));
        href
    };
    let mut out: Vec<MarketplaceChip> = tallies
        .iter()
        .map(|tally| MarketplaceChip {
            href: query(&tally.id),
            is_active: !plan.all_marketplaces && tally.id == selected,
            id: tally.id.clone(),
            name: tally.name.clone(),
            skills: tally.skills,
        })
        .collect();
    out.push(MarketplaceChip {
        id: ALL_MARKETPLACES.to_owned(),
        name: "All".to_owned(),
        skills: tallies.iter().map(|t| t.skills).sum(),
        href: query(ALL_MARKETPLACES),
        is_active: plan.all_marketplaces,
    });
    out
}

// Why: what the page says about the rows, as opposed to what it reads off
// them — the chip row, the unresolved names, the tiles and the status line.
// Gathering them in one place keeps `load` a list of steps.
pub(super) struct Derived {
    pub(super) chips: Vec<MarketplaceChip>,
    pub(super) collisions: Vec<Collision>,
    pub(super) unbound: usize,
    pub(super) conflicting: usize,
    pub(super) kpis: Kpis,
    pub(super) snapshot_state: &'static str,
    pub(super) onboarding: Option<Onboarding>,
    pub(super) synced_at: Option<String>,
    pub(super) selected_marketplace_name: String,
}

impl Derived {
    pub(super) fn build(
        collected: &Collected,
        plan: &Plan,
        tallies: &[MarketplaceTally],
        selected: &str,
        retained: &Retained,
    ) -> Self {
        let count = |status: &'static str| {
            collected
                .rows
                .iter()
                .filter(|row| row.usage_status == status)
                .count()
        };
        let kpis = groups::kpis(&collected.rows, retained.figures.as_ref());
        Self {
            chips: chips(tallies, plan, selected),
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
            synced_at: retained.health.generated_at.map(|at| at.to_rfc3339()),
            selected_marketplace_name: tallies
                .iter()
                .find(|tally| tally.id == selected)
                .map_or_else(String::new, |tally| tally.name.clone()),
            kpis,
        }
    }
}

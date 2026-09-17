//! One inventory skill as the dense table shows it: figures read from its
//! retained feedback snapshot, the audience that receives it, and a single
//! health verdict a reader can scan by colour.
//!
//! A skill without a snapshot is *unmeasured*, not zero: attribution is
//! verified evidence, and an entry that is not bound to a managed resource has
//! no scope a snapshot could be retained under.

pub use super::cells::{RowVersion, live_for, p95_cell, scores_for, short_hash, version_href};
use crate::error::AdminResult;
use crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use crate::handlers::ssr::format::{format_cost, short_num};
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::repositories::analysis::marketplace_versions::SkillLiveRow;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use crate::repositories::evaluation::skill_scores::SkillScoreFigures;
use crate::repositories::sync::marketplace_hash::skill_key;
use serde::Serialize;
use systemprompt::marketplace::inventory::InventoryEntry;
use systemprompt::marketplace::managed::ResourceSummary;
use systemprompt::models::feedback::inventory::InventoryAvailability;
use systemprompt_web_shared::PluginId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Ok,
    Warn,
    Err,
    Muted,
}

impl Health {
    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Err => "err",
            Self::Muted => "muted",
        }
    }
}

/// The inputs the health verdict is derived from, separated from the row so
/// the rule can be exercised without an inventory entry.
#[derive(Debug, Clone, Copy)]
pub struct HealthInputs {
    pub availability: InventoryAvailability,
    pub bound: bool,
    pub measured: bool,
    pub invocations: i64,
    pub requests: i64,
    pub failed_requests: i64,
    pub users_with_access: usize,
}

#[must_use]
pub const fn health_of(input: &HealthInputs) -> (Health, &'static str) {
    match input.availability {
        InventoryAvailability::Conflicting => return (Health::Err, "conflicting name"),
        InventoryAvailability::Unavailable => return (Health::Err, "unavailable"),
        InventoryAvailability::Withdrawn => return (Health::Err, "withdrawn"),
        InventoryAvailability::Available => {},
    }
    if input.requests > 0 && input.failed_requests * 10 > input.requests {
        return (Health::Err, "failing requests");
    }
    if input.users_with_access == 0 {
        return (Health::Warn, "no audience");
    }
    if !input.bound {
        return (Health::Warn, "unbound");
    }
    if !input.measured {
        return (Health::Muted, "snapshot pending");
    }
    if input.invocations == 0 {
        return (Health::Warn, "no attributed use");
    }
    (Health::Ok, "healthy")
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryRow {
    pub entry: InventoryEntry,
    pub resource_key: String,
    pub plugin_id: Option<PluginId>,
    pub plugin_name: Option<String>,
    pub marketplaces: Vec<MarketplaceRef>,
    pub catalog_href: Option<String>,
    pub drilldown_href: String,
    // Why: the `plugin:skill` key the facts and the judge attribution use;
    // `None` for an entry no plugin carries.
    pub skill_key: Option<String>,
    pub scorecard: ScorecardView,
    pub scored_sessions: i64,
    pub figures: SnapshotFigures,
    pub users_with_access: usize,
    pub users: Option<i64>,
    pub cost: String,
    pub p95: String,
    pub p95_measured: bool,
    pub tokens_in: String,
    pub tokens_out: String,
    pub version: Option<RowVersion>,
    pub health: Health,
    pub health_tone: &'static str,
    pub health_reason: &'static str,
    pub usage_status: &'static str,
    pub bind_candidates: Vec<ResourceSummary>,
}

#[derive(Debug)]
pub struct InventoryRowInputs<'a> {
    pub item: PortfolioEntry,
    pub plugin: Option<(&'a str, &'a str)>,
    pub marketplaces: Vec<MarketplaceRef>,
    pub users_with_access: usize,
    pub managed: &'a [ResourceSummary],
    pub live: Option<&'a SkillLiveRow>,
    pub scores: Option<&'a SkillScoreFigures>,
    pub window: (&'a str, &'a str),
}

// Why: an unbound configured entry may match several imported managed
// resources by name; the row offers those as bind candidates.
struct Binding {
    candidates: Vec<ResourceSummary>,
}

fn binding(entry: &InventoryEntry, managed: &[ResourceSummary]) -> Binding {
    let same_key = |r: &&ResourceSummary| {
        r.kind.as_str() == entry.kind && r.resource_key == entry.resource_key
    };
    Binding {
        candidates: if entry.resource_id.is_none() {
            managed.iter().filter(same_key).cloned().collect()
        } else {
            Vec::new()
        },
    }
}

// Why: the name always links to the skill's own page, even with nothing
// attributed yet — the reader learns more from that page's empty state than
// from a name that does not respond. The page is keyed the way the facts
// are (`plugin:skill`); a bound entry also narrows it to the revision-keyed
// managed resource. Without a plugin there is no fact key, so the link
// falls back to the list searched for the name.
fn conversations_href(
    entry: &InventoryEntry,
    skill_key: Option<&str>,
    window: (&str, &str),
) -> String {
    skill_key.map_or_else(
        || {
            format!(
                "{ANALYSIS_SKILLS_URL}?search={}",
                urlencoding::encode(&entry.resource_key)
            )
        },
        |key| super::super::drilldown_url(key, entry.resource_id.as_ref(), window),
    )
}

const fn usage_status(item: &PortfolioEntry, figures: Option<&SnapshotFigures>) -> &'static str {
    match (
        figures,
        &item.snapshot,
        &item.entry.resource_id,
        item.entry.availability,
    ) {
        (_, _, _, InventoryAvailability::Conflicting) => "conflicting",
        (Some(_), _, _, _) => "measured",
        (None, Some(_), _, _) => "suppressed",
        (None, None, Some(_), _) => "pending",
        (None, None, None, _) => "unbound",
    }
}

fn scorecard(scores: Option<&SkillScoreFigures>, drilldown_href: String) -> ScorecardView {
    ScorecardView::with_prior(
        scores.map_or_else(Scores::default, |f| Scores {
            intent: f.intent,
            quality: f.quality,
            speed: f.speed,
            cost: f.cost,
        }),
        Scores {
            quality: scores.and_then(|f| f.quality_prior),
            ..Scores::default()
        },
        Some(drilldown_href),
        scores.map_or_else(
            || "No judged session invoked this skill in the window".to_owned(),
            |f| {
                format!(
                    "Mean judge scores over {} judged sessions that invoked this skill; opens the skill's page",
                    f.scored_sessions
                )
            },
        ),
    )
}

fn catalog_href(entry: &InventoryEntry) -> Option<String> {
    match entry.kind.as_str() {
        "skill" | "plugin" => Some(format!(
            "/admin/catalog/{}s/{}",
            entry.kind,
            urlencoding::encode(&entry.resource_key)
        )),
        _ => None,
    }
}

impl InventoryRow {
    pub fn build(input: InventoryRowInputs<'_>) -> AdminResult<Self> {
        let InventoryRowInputs {
            item,
            plugin,
            marketplaces,
            users_with_access,
            managed,
            live,
            scores,
            window,
        } = input;
        let figures = item
            .snapshot
            .as_ref()
            .filter(|s| s.resource_id == item.entry.resource_id)
            .map(SnapshotFigures::of)
            .transpose()?
            .flatten();
        let usage_status = usage_status(&item, figures.as_ref());
        let figures = figures.unwrap_or_default();
        let entry = item.entry;
        let binding = binding(&entry, managed);
        let (p95, p95_measured) = p95_cell(live, figures.p95_micros);
        let (health, health_reason) = health_of(&HealthInputs {
            availability: entry.availability,
            bound: entry.resource_id.is_some(),
            measured: figures.measured,
            invocations: figures.invocations,
            requests: figures.requests,
            failed_requests: figures.failed_requests,
            users_with_access,
        });
        let skill_key = plugin.map(|p| skill_key(p.0, &entry.resource_key));
        let drilldown_href = conversations_href(&entry, skill_key.as_deref(), window);
        let scorecard = scorecard(scores, drilldown_href.clone());
        let catalog_href = catalog_href(&entry);
        Ok(Self {
            resource_key: entry.resource_key.clone(),
            plugin_id: plugin.map(|p| PluginId::new(p.0)),
            plugin_name: plugin.map(|p| p.1.to_owned()),
            catalog_href,
            drilldown_href,
            skill_key,
            scorecard,
            scored_sessions: scores.map_or(0, |f| f.scored_sessions),
            users_with_access,
            users: live.map(|l| l.users).or(figures.distinct_users),
            cost: if figures.priced_requests > 0 {
                format_cost(figures.cost_microdollars)
            } else {
                "—".to_owned()
            },
            p95,
            p95_measured,
            tokens_in: count(figures.input_tokens),
            tokens_out: count(figures.output_tokens),
            health,
            health_tone: health.tone(),
            health_reason,
            usage_status,
            bind_candidates: binding.candidates,
            marketplaces,
            figures,
            version: None,
            entry,
        })
    }
}

fn count(n: i64) -> String {
    if n == 0 { "0".to_owned() } else { short_num(n) }
}

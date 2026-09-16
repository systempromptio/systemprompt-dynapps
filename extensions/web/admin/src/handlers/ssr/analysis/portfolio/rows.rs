//! One inventory skill as the dense table shows it: figures read from its
//! retained feedback snapshot, the audience that receives it, and a single
//! health verdict a reader can scan by colour.
//!
//! A skill without a snapshot is *unmeasured*, not zero: attribution is
//! verified evidence, and an entry that is not bound to a managed resource has
//! no scope a snapshot could be retained under.

use crate::error::AdminResult;
use crate::handlers::ssr::format::{format_cost, format_duration_ms, short_num};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use crate::repositories::analysis::publication_heads::PublicationHead;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use serde::Serialize;
use systemprompt::identifiers::ResourceRevisionId;
use systemprompt::marketplace::inventory::{InstallationCoverage, InventoryEntry};
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
    pub figures: SnapshotFigures,
    pub users_with_access: usize,
    pub cost: String,
    pub p95: String,
    pub tokens_in: String,
    pub tokens_out: String,
    pub installation: Option<InstallationCoverage>,
    pub published_generation: Option<i64>,
    pub published_revision: Option<ResourceRevisionId>,
    pub latest_revision: Option<ResourceRevisionId>,
    pub unpublished_revision: bool,
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
    pub heads: &'a [PublicationHead],
    pub window: (&'a str, &'a str),
}

/// What the bridge serves for an entry against what the inventory last
/// captured; `unpublished` is the gap the next sync pass closes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedState {
    pub generation: Option<i64>,
    pub revision: Option<ResourceRevisionId>,
    pub unpublished: bool,
}

#[must_use]
pub fn published_state(entry: &InventoryEntry, heads: &[PublicationHead]) -> PublishedState {
    let head = entry
        .resource_id
        .as_ref()
        .and_then(|id| heads.iter().find(|h| &h.resource_id == id));
    let revision = head.and_then(|h| h.published_revision().cloned());
    PublishedState {
        generation: head.map(|h| h.generation),
        unpublished: entry.latest_revision_id.is_some() && entry.latest_revision_id != revision,
        revision,
    }
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

// Why: the name always links to the conversations that used the skill, even
// with nothing attributed yet — the reader learns more from that page's empty
// state than from a name that does not respond. A bound entry links by its
// managed resource, which is revision-keyed; an unbound one has only its skill
// identity to go on.
fn conversations_href(entry: &InventoryEntry, window: (&str, &str)) -> String {
    entry.resource_id.as_ref().map_or_else(
        || {
            format!(
                "/admin/analysis/skills?skill={}&start={}&end={}",
                urlencoding::encode(&entry.resource_key),
                urlencoding::encode(window.0),
                urlencoding::encode(window.1)
            )
        },
        |resource| super::super::drilldown_url(resource, window),
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

impl InventoryRow {
    pub fn build(input: InventoryRowInputs<'_>) -> AdminResult<Self> {
        let InventoryRowInputs {
            item,
            plugin,
            marketplaces,
            users_with_access,
            managed,
            heads,
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
        let published = published_state(&entry, heads);
        let (health, health_reason) = health_of(&HealthInputs {
            availability: entry.availability,
            bound: entry.resource_id.is_some(),
            measured: figures.measured,
            invocations: figures.invocations,
            requests: figures.requests,
            failed_requests: figures.failed_requests,
            users_with_access,
        });
        let drilldown_href = conversations_href(&entry, window);
        let catalog_href = match entry.kind.as_str() {
            "skill" | "plugin" => Some(format!(
                "/admin/catalog/{}s/{}",
                entry.kind,
                urlencoding::encode(&entry.resource_key)
            )),
            _ => None,
        };
        Ok(Self {
            resource_key: entry.resource_key.clone(),
            plugin_id: plugin.map(|p| PluginId::new(p.0)),
            plugin_name: plugin.map(|p| p.1.to_owned()),
            catalog_href,
            drilldown_href,
            users_with_access,
            cost: if figures.priced_requests > 0 {
                format_cost(figures.cost_microdollars)
            } else {
                "—".to_owned()
            },
            p95: figures.p95_micros.map_or_else(
                || "—".to_owned(),
                |micros| format!("≤ {}", format_duration_ms((micros / 1000) as i64)),
            ),
            tokens_in: count(figures.input_tokens),
            tokens_out: count(figures.output_tokens),
            health,
            health_tone: health.tone(),
            health_reason,
            usage_status,
            bind_candidates: binding.candidates,
            marketplaces,
            figures,
            installation: item.installation,
            published_generation: published.generation,
            published_revision: published.revision,
            latest_revision: entry.latest_revision_id.clone(),
            unpublished_revision: published.unpublished,
            entry,
        })
    }
}

fn count(n: i64) -> String {
    if n == 0 { "0".to_owned() } else { short_num(n) }
}

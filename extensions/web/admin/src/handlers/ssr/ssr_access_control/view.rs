//! Template contexts for `/admin/access-control` and its sync page.
//!
//! The Rules tab is entity-centric: one row per governed entity under a
//! heading per kind, its bands as chips, its why in full, and whether code
//! and database agree about it. The audience grid — every role, group and
//! project against every entity, each cell the resolver's own answer — and
//! the one-person check are their own tabs. `can_write` is the MANAGE tier;
//! a project manager reads every tab and changes nothing.

use serde::Serialize;

use super::person::PersonCheckView;
use crate::handlers::ssr::ssr_users::detail::AccessTabView;
use crate::handlers::ssr::sync_plane::PlaneCardView;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::access_control::drift::DriftCounts;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcStatsView {
    pub groups: usize,
    pub projects: usize,
    pub users: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcKpiView {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

// Why: One band's subjects on one entity, e.g. `group: india-devs, uk`.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct BandChipsView {
    pub band: String,
    pub label: &'static str,
    pub values: Vec<String>,
}

// Why: One rule row, for the expanded detail under an entity.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcRuleView {
    pub band: String,
    pub band_label: &'static str,
    pub subject: String,
    pub access: String,
    pub access_tone: &'static str,
    pub source: String,
    pub source_tone: &'static str,
    pub justification: String,
    pub expires_at: Option<String>,
    pub expires_soon: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcEntityView {
    pub entity_type: String,
    pub entity_type_label: &'static str,
    pub entity_id: String,
    // Why: what the row is called — a route's declared name, a plugin's
    // title — with the id demoted to a subtitle. `entity_sub` is the one
    // line of context a generated id cannot give (pattern → provider).
    pub entity_label: String,
    pub entity_sub: Option<String>,
    pub labelled: bool,
    pub default_open: bool,
    pub default_label: &'static str,
    pub allow: Vec<BandChipsView>,
    pub deny: Vec<BandChipsView>,
    pub why: String,
    pub has_why: bool,
    pub state: &'static str,
    pub state_tone: &'static str,
    pub in_sync: bool,
    pub sync_url: String,
    pub rules: Vec<AcRuleView>,
    pub rule_count: usize,
    pub resolution: String,
    pub expiring_soon: bool,
}

// Why: one heading per entity kind, in a fixed order, so the table reads
// as a catalogue rather than a ledger dump.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcEntityGroupView {
    pub kind: String,
    pub kind_label: &'static str,
    pub count: usize,
    pub rows: Vec<AcEntityView>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AcEntitiesView {
    pub groups: Vec<AcEntityGroupView>,
    pub total: usize,
    pub kpis: Vec<AcKpiView>,
    pub entity_options: Vec<AcOptionView>,
    pub band_options: Vec<AcOptionView>,
    pub state_options: Vec<AcOptionView>,
    pub expiring_only: bool,
    pub search: String,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    pub capped: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DriftBannerView {
    pub counts: DriftCounts,
    pub url: &'static str,
    pub declared: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcAudienceColumnView {
    pub subject: String,
    pub label: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcAudienceCellView {
    pub allow: bool,
    pub effective: String,
    pub layer: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcAudienceRowView {
    pub entity_type: String,
    pub entity_type_label: &'static str,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_sub: Option<String>,
    pub cells: Vec<AcAudienceCellView>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AudienceGridView {
    pub columns: Vec<AcAudienceColumnView>,
    pub rows: Vec<AcAudienceRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct AccessControlPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub stats: AcStatsView,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub on_rules: bool,
    pub on_audience: bool,
    pub on_person: bool,
    // Why: each tab's body is set only for that tab; the others stay empty.
    pub sync: Option<PlaneCardView>,
    pub docs_url: &'static str,
    pub sync_url: &'static str,
    pub drift: Option<DriftBannerView>,
    pub declared_unreadable: Option<String>,
    pub entities: AcEntitiesView,
    pub audience: AudienceGridView,
    pub person: Option<PersonCheckView>,
    // Why: named `access` at the page root because the shared
    // `components/user-access` partial reads `@root.access`.
    pub access: Option<AccessTabView>,
}

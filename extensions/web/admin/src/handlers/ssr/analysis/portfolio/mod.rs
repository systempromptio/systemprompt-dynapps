//! Skills: every inventory skill with the figures its retained feedback
//! snapshot holds for the selected window, grouped by the marketplace that
//! serves it and stamped with that marketplace's version hash.
//!
//! The page opens on every marketplace at once — the grouped marketplace →
//! plugin → skill view is the only place a skill carried by two marketplaces
//! is visibly carried by both — and the Marketplace field narrows it.
//!
//! Figures come from retained reporting data only: the snapshot for
//! invocations, requests and spend; the normalized facts for who invoked a
//! skill and a real latency percentile. The headline tiles are the
//! organisation-wide snapshot and include traffic not yet attributed to any
//! resource; a row is measured only once its entry is bound to a managed
//! resource and a snapshot exists for that scope. Unbound and pending rows
//! say so instead of showing a zero they cannot substantiate.

use crate::error::{AdminHtmlResult, AdminResult};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::extract::{Query, State};
use axum::response::Response;
use axum::{Extension, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotHealth, SnapshotRangeJob};
use systemprompt::marketplace::inventory::{InstallationCoverageStatus, InventoryStatus};

pub(crate) mod actions;
pub(crate) mod cells;
mod collect;
pub(crate) mod collisions;
mod derived;
mod evidence;
pub(crate) mod export;
pub(crate) mod groups;
mod plan;
mod range;
pub(crate) mod rows;
use crate::handlers::ssr::page::Page;
use collect::{Collected, collect_rows};
use collisions::Collision;
use derived::{Derived, InventoryMarketplaceOption, Onboarding, SelectedMarketplace};
use evidence::Evidence;
use groups::{Kpis, MarketplaceGroup, MarketplaceTally};
use plan::{Plan, plan};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortfolioQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<u32>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub marketplace: Option<String>,
    pub columns: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PortfolioPage {
    page: &'static str,
    title: &'static str,
    kind: String,
    groups: Vec<MarketplaceGroup>,
    kpis: Kpis,
    marketplaces: Vec<MarketplaceRef>,
    filter_marketplace: String,
    marketplace_count: usize,
    start: String,
    end: String,
    search: String,
    days: u32,
    generated_at: String,
    totals: Option<FeedbackSnapshot>,
    health: SnapshotHealth,
    inventory: InventoryStatus,
    installations: InstallationCoverageStatus,
    pending_jobs: usize,
    range_jobs: Vec<SnapshotRangeJob>,
    unknown_usage: Option<i64>,
    unbound: usize,
    conflicting: usize,
    marketplace_options: Vec<InventoryMarketplaceOption>,
    plugin_count: usize,
    audience: usize,
    selected_marketplace: String,
    selected_marketplace_name: String,
    all_view: bool,
    columns_all: bool,
    collisions: Vec<Collision>,
    collision_count: usize,
    // Why: RFC 3339, or `None` before the first inventory pass — the meta line
    // says "never synced" rather than handing `relativeTime` a sentence.
    synced_at: Option<String>,
    snapshot_state: &'static str,
    onboarding: Option<Onboarding>,
    export: crate::export::ExportView,
}

pub(crate) async fn page(
    shell: Page,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminHtmlResult<Response> {
    let data = load(&managed, &pool, &shell.user, query).await?;
    Ok(super::super::render_typed_page(
        &shell.engine,
        "analysis-skills",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

pub(crate) async fn json(
    Extension(user): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminResult<Json<PortfolioPage>> {
    Ok(Json(load(&managed, &pool, &user, query).await?))
}

pub(super) struct Retained {
    pub(super) health: SnapshotHealth,
    pub(super) inventory: InventoryStatus,
    pub(super) installations: InstallationCoverageStatus,
    pub(super) unknown_usage: Option<i64>,
    pub(super) figures: Option<SnapshotFigures>,
}

impl Retained {
    async fn read(managed: &ManagedState, totals: Option<&FeedbackSnapshot>) -> AdminResult<Self> {
        Ok(Self {
            health: managed.snapshots.health(&managed.owner).await?,
            inventory: managed.repository.inventory_status(&managed.owner).await?,
            installations: managed
                .repository
                .installation_coverage_status(&managed.owner)
                .await?,
            unknown_usage: totals.map(|snapshot| {
                snapshot.metrics.invocations - snapshot.metrics.verified_invocations
            }),
            figures: totals.map(SnapshotFigures::of).transpose()?.flatten(),
        })
    }
}

// Why: the picker must offer every marketplace the inventory reaches, so the
// tally is taken before the view narrows to one of them.
pub(super) fn narrow_to_marketplace(
    collected: &mut Collected,
    plan: &Plan,
) -> (Vec<MarketplaceTally>, String) {
    let tallies = groups::tally_marketplaces(&collected.rows);
    let selected = plan.filter_marketplace.clone();
    if !plan.all_marketplaces && !selected.is_empty() {
        collected
            .rows
            .retain(|row| groups::in_marketplace(row, &selected));
    }
    (tallies, selected)
}

// Why: a custom range has no retained organisation-wide snapshot, so the
// headline tiles wait on a job that computes one.
async fn organisation_totals(
    managed: &ManagedState,
    plan: &Plan,
    collected: &mut Collected,
    now: chrono::DateTime<Utc>,
) -> AdminResult<Option<FeedbackSnapshot>> {
    if !plan.range.custom {
        return Ok(managed
            .snapshots
            .snapshot(&managed.owner, None, plan.range.window)
            .await?);
    }
    let job = range::custom_job(managed, None, &plan.range, now).await?;
    let snapshot = job.result.clone();
    if matches!(job.state.as_str(), "pending" | "leased") {
        collected.pending_jobs += 1;
    }
    collected.range_jobs.push(job);
    Ok(snapshot)
}

async fn load(
    managed: &ManagedState,
    pool: &PgPool,
    user: &UserContext,
    query: PortfolioQuery,
) -> AdminResult<PortfolioPage> {
    export::require_admin(user)?;
    let now = Utc::now();
    let plan = plan(query, now)?;
    let evidence = Evidence::gather(managed, pool, &plan.index, plan.range.retained()).await?;
    let mut collected = collect_rows(managed, &plan, &evidence, now).await?;
    let (tallies, selected) = narrow_to_marketplace(&mut collected, &plan);
    let totals = organisation_totals(managed, &plan, &mut collected, now).await?;
    let retained = Retained::read(managed, totals.as_ref()).await?;
    let marketplace = SelectedMarketplace {
        tallies: &tallies,
        id: &selected,
        audience: evidence.audience_of(&plan, &selected),
    };
    let mut derived = Derived::build(&collected, &plan, marketplace, &retained);
    (derived.kpis.scored_sessions, derived.kpis.scorecard) = evidence.score_kpis();
    let groups = groups::group_rows(
        collected.rows,
        |id| evidence.audience.users_reaching_marketplace(id),
        &evidence.versions,
    );
    let export = export::export_view(&plan);
    Ok(PortfolioPage {
        page: "analysis-skills",
        title: "Skills",
        kind: plan.kind,
        groups,
        kpis: derived.kpis,
        marketplaces: plan.index.marketplace_refs(),
        marketplace_count: tallies.iter().filter(|t| t.id != groups::UNPLACED).count(),
        filter_marketplace: plan.filter_marketplace,
        start: plan.range.from.to_string(),
        end: plan.range.to.to_string(),
        search: plan.search,
        days: plan.range.window,
        generated_at: retained.health.generated_at.map_or_else(
            || "Awaiting first snapshot".to_owned(),
            |at| at.to_rfc3339(),
        ),
        totals,
        health: retained.health,
        inventory: retained.inventory,
        installations: retained.installations,
        pending_jobs: collected.pending_jobs,
        range_jobs: collected.range_jobs,
        unknown_usage: retained.unknown_usage,
        unbound: derived.unbound,
        conflicting: derived.conflicting,
        selected_marketplace_name: derived.selected_marketplace_name,
        selected_marketplace: selected,
        all_view: plan.all_marketplaces,
        columns_all: plan.columns_all,
        marketplace_options: derived.marketplace_options,
        plugin_count: derived.plugin_count,
        audience: derived.audience,
        collision_count: derived.collisions.len(),
        collisions: derived.collisions,
        synced_at: derived.synced_at,
        snapshot_state: derived.snapshot_state,
        onboarding: derived.onboarding,
        export,
    })
}

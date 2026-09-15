//! Resource effectiveness: every inventory skill, grouped by the marketplace
//! and plugin that deliver it, with the figures its retained feedback snapshot
//! holds for the selected window.
//!
//! Every number on the page is read from a retained snapshot; nothing is
//! recomputed from raw events. The headline tiles are the organisation-wide
//! snapshot and include traffic not yet attributed to any resource; a row is
//! measured only once its entry is bound to a managed resource and a snapshot
//! exists for that scope. Unbound and pending rows say so instead of showing
//! a zero they cannot substantiate.

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
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
mod evidence;
pub(crate) mod groups;
mod plan;
mod range;
pub(crate) mod rows;
use evidence::{Evidence, expand_rows};
use groups::{Kpis, MarketplaceGroup};
use plan::{Plan, list_inventory, plan};
use rows::InventoryRow;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortfolioQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<u32>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub marketplace: Option<String>,
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
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminHtmlResult<Response> {
    let data = load(&managed, &pool, &user, query).await?;
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-portfolio",
        &data,
        &user,
        &marketplace,
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

struct Collected {
    rows: Vec<InventoryRow>,
    pending_jobs: usize,
    range_jobs: Vec<SnapshotRangeJob>,
}

async fn collect_rows(
    managed: &ManagedState,
    plan: &Plan,
    evidence: &Evidence<'_>,
    now: chrono::DateTime<Utc>,
) -> AdminResult<Collected> {
    let items = list_inventory(managed, plan).await?;
    let needle = plan.search.to_lowercase();
    let window = (plan.range.from.to_string(), plan.range.to.to_string());
    let mut collected = Collected {
        rows: Vec::new(),
        pending_jobs: 0,
        range_jobs: Vec::new(),
    };
    for mut item in items {
        if !needle.is_empty() && !item.entry.resource_key.to_lowercase().contains(&needle) {
            continue;
        }
        if plan.range.custom
            && let Some(resource) = item.entry.resource_id.as_ref()
        {
            let job = range::custom_job(managed, Some(resource), &plan.range, now).await?;
            item.snapshot.clone_from(&job.result);
            if matches!(job.state.as_str(), "pending" | "leased") {
                collected.pending_jobs += 1;
            }
            collected.range_jobs.push(job);
        }
        collected
            .rows
            .extend(expand_rows(item, evidence, (&window.0, &window.1))?);
    }
    if !plan.filter_marketplace.is_empty() {
        collected.rows.retain(|row| {
            row.marketplaces
                .iter()
                .any(|m| m.id == plan.filter_marketplace)
        });
    }
    Ok(collected)
}

struct Retained {
    health: SnapshotHealth,
    inventory: InventoryStatus,
    installations: InstallationCoverageStatus,
    unknown_usage: Option<i64>,
    figures: Option<SnapshotFigures>,
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

async fn load(
    managed: &ManagedState,
    pool: &PgPool,
    user: &UserContext,
    query: PortfolioQuery,
) -> AdminResult<PortfolioPage> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for organizational resource analysis".to_owned(),
        ));
    }
    let now = Utc::now();
    let plan = plan(query, now)?;
    let evidence = Evidence::gather(managed, pool, &plan.index).await?;
    let mut collected = collect_rows(managed, &plan, &evidence, now).await?;
    let unbound = collected
        .rows
        .iter()
        .filter(|r| r.usage_status == "unbound")
        .count();
    let conflicting = collected
        .rows
        .iter()
        .filter(|r| r.usage_status == "conflicting")
        .count();
    let totals = if plan.range.custom {
        let job = range::custom_job(managed, None, &plan.range, now).await?;
        let snapshot = job.result.clone();
        if matches!(job.state.as_str(), "pending" | "leased") {
            collected.pending_jobs += 1;
        }
        collected.range_jobs.push(job);
        snapshot
    } else {
        managed
            .snapshots
            .snapshot(&managed.owner, None, plan.range.window)
            .await?
    };
    let retained = Retained::read(managed, totals.as_ref()).await?;
    let kpis = groups::kpis(&collected.rows, retained.figures.as_ref());
    let groups = groups::group_rows(collected.rows, |id| {
        evidence.audience.users_reaching_marketplace(id)
    });
    Ok(PortfolioPage {
        page: "analysis-portfolio",
        title: "Resource effectiveness",
        kind: plan.kind,
        groups,
        kpis,
        marketplaces: plan
            .index
            .marketplaces
            .iter()
            .map(|m| MarketplaceRef {
                id: m.id.as_str().to_owned(),
                name: m.name.clone(),
            })
            .collect(),
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
        unbound,
        conflicting,
    })
}

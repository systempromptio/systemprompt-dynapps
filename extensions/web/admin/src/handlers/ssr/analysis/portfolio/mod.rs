//! Bounded canonical inventory with retained snapshot usage and current catalog
//! placement.

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::groups::GroupSummary;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::{Extension, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotHealth, SnapshotRangeJob};
use systemprompt::identifiers::InventoryEntryId;
use systemprompt::marketplace::inventory::{InstallationCoverageStatus, InventoryStatus};

pub(crate) mod actions;
mod assembly;
mod collection;
mod evidence;
mod plan;
use collection::collect_rows;
use plan::{Plan, plan};
pub(crate) mod groups;
mod range;
mod row_display;
pub(crate) mod rows;
use evidence::{Evidence, expand_rows};
use groups::{Kpis, MarketplaceGroup};
use rows::SkillRow;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortfolioQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<u32>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub marketplace: Option<String>,
    pub group: Option<String>,
    pub after: Option<InventoryEntryId>,
}

#[derive(Serialize)]
pub(crate) struct PortfolioPage {
    page: &'static str,
    title: &'static str,
    kind: String,
    groups: Vec<MarketplaceGroup>,
    kpis: Kpis,
    marketplaces: Vec<MarketplaceRef>,
    user_groups: Vec<GroupSummary>,
    filter_marketplace: String,
    filter_group: String,
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
    operation_key: String,
    next_cursor: Option<InventoryEntryId>,
    next_href: Option<String>,
    metrics_notice: String,
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
    let mut evidence = Evidence::gather(&plan.index);
    let (rows, mut pending_jobs, next_cursor, mut range_jobs) =
        collect_rows(managed, &plan, &mut evidence, now).await?;
    let totals = if !plan.filter_group.is_empty()
        || !plan.filter_marketplace.is_empty()
        || !plan.search.is_empty()
    {
        None
    } else if plan.range.custom {
        let job = range::custom_job(managed, None, &plan.range, now).await?;
        let snapshot = job.result.clone();
        if matches!(job.state.as_str(), "pending" | "leased") {
            pending_jobs += 1;
        }
        range_jobs.push(job);
        snapshot
    } else {
        managed
            .snapshots
            .snapshot(&managed.owner, None, plan.range.window)
            .await?
    };
    let health = managed.snapshots.health(&managed.owner).await?;
    let inventory = managed.repository.inventory_status(&managed.owner).await?;
    let installations = managed
        .repository
        .installation_coverage_status(&managed.owner)
        .await?;
    Ok(assembly::assemble(assembly::LoadedPage {
        plan,
        rows,
        pending_jobs,
        next_cursor,
        range_jobs,
        totals,
        health,
        inventory,
        installations,
        user_groups: crate::repositories::groups::crud::list_group_summaries(pool).await?,
    }))
}

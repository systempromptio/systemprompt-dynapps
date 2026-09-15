//! Portfolio rendering consumes retained snapshots and canonical observed
//! inventory.

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories::analysis::portfolio_snapshots::PortfolioFilter;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::Query;
use axum::response::Response;
use axum::{Extension, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotHealth};
use systemprompt::identifiers::InventoryEntryId;
use systemprompt::marketplace::inventory::{InstallationCoverageStatus, InventoryStatus};

mod range;
mod rows;
use rows::ResourceRow;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PortfolioQuery {
    pub start: Option<String>,
    pub end: Option<String>,
    pub days: Option<u32>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub after: Option<InventoryEntryId>,
    pub limit: Option<u32>,
}
#[derive(Serialize)]
pub(crate) struct PortfolioPage {
    page: &'static str,
    title: &'static str,
    kind: String,
    rows: Vec<ResourceRow>,
    start: String,
    end: String,
    search: String,
    generated_at: String,
    totals: Option<FeedbackSnapshot>,
    health: SnapshotHealth,
    inventory: InventoryStatus,
    installations: InstallationCoverageStatus,
    next_cursor: Option<InventoryEntryId>,
    next_href: Option<String>,
    pending_jobs: usize,
    unknown_usage: Option<i64>,
    days: u32,
}
pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    Query(query): Query<PortfolioQuery>,
) -> AdminHtmlResult<Response> {
    let data = load(&managed, &user, query).await?;
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
    Query(query): Query<PortfolioQuery>,
) -> AdminResult<Json<PortfolioPage>> {
    Ok(Json(load(&managed, &user, query).await?))
}

async fn load(
    managed: &ManagedState,
    user: &UserContext,
    query: PortfolioQuery,
) -> AdminResult<PortfolioPage> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for organizational resource analysis".to_owned(),
        ));
    }
    let now = Utc::now();
    let range = range::Range::parse(&query, now.date_naive())?;
    let kind = query.kind.unwrap_or_else(|| "skills".to_owned());
    let filter_kind = match kind.as_str() {
        "all" => None,
        "skills" => Some("skill"),
        "plugins" => Some("plugin"),
        "marketplaces" => Some("marketplace"),
        "rules" => Some("rule"),
        "hooks" => Some("hook"),
        "artifacts" => Some("artifact"),
        "agents" => Some("agent"),
        "mcp" => Some("mcp"),
        _ => return Err(AdminError::BadRequest("Invalid inventory kind".to_owned())),
    };
    let search = query.search.unwrap_or_default();
    let limit = query.limit.unwrap_or(50);
    if !(1..=50).contains(&limit) {
        return Err(AdminError::BadRequest(
            "Portfolio page size must be1–50".to_owned(),
        ));
    }
    let items = managed
        .portfolio
        .list(
            &managed.owner,
            &PortfolioFilter {
                after: query.after.as_ref(),
                kind: filter_kind,
                search: &search,
                window: range.window,
                limit,
            },
        )
        .await?;
    let next_cursor = if items.len() == limit as usize {
        items.last().map(|item| item.entry.entry_id.clone())
    } else {
        None
    };
    let next_href = next_cursor.as_ref().map(|id| {
        format!(
            "?kind={}&start={}&end={}&search={}&after={}",
            urlencoding::encode(&kind),
            range.from,
            range.to,
            urlencoding::encode(&search),
            urlencoding::encode(id.as_str())
        )
    });
    let mut pending_jobs = 0;
    let mut result = Vec::new();
    for mut item in items {
        if range.custom {
            if let Some(resource) = item.entry.resource_id.as_ref() {
                item.snapshot =
                    range::custom_snapshot(managed, Some(resource), &range, now).await?;
                if item.snapshot.is_none() {
                    pending_jobs += 1;
                }
            }
        }
        result.push(ResourceRow::from_entry(item)?);
    }
    let totals = if range.custom {
        let snapshot = range::custom_snapshot(managed, None, &range, now).await?;
        if snapshot.is_none() {
            pending_jobs += 1;
        }
        snapshot
    } else {
        managed
            .snapshots
            .snapshot(&managed.owner, None, range.window)
            .await?
    };
    let health = managed.snapshots.health(&managed.owner).await?;
    let inventory = managed.repository.inventory_status(&managed.owner).await?;
    let installations = managed
        .repository
        .installation_coverage_status(&managed.owner)
        .await?;
    let unknown_usage = totals
        .as_ref()
        .map(|snapshot| snapshot.metrics.invocations - snapshot.metrics.verified_invocations);
    Ok(PortfolioPage {
        page: "analysis-portfolio",
        title: "Resource effectiveness",
        kind,
        rows: result,
        start: range.from.to_string(),
        end: range.to.to_string(),
        search,
        generated_at: health.generated_at.map_or_else(
            || "Awaiting first snapshot".to_owned(),
            |at| at.to_rfc3339(),
        ),
        totals,
        health,
        inventory,
        installations,
        next_cursor,
        next_href,
        pending_jobs,
        unknown_usage,
        days: range.window,
    })
}

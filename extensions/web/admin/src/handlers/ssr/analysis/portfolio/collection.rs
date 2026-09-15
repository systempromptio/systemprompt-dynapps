//! One canonical cursor page with bounded explicit binding candidates.
use super::{Evidence, Plan, PortfolioResourceRow, expand_rows, range};
use crate::error::AdminResult;
use crate::repositories::analysis::portfolio_snapshots::PortfolioFilter;
use crate::routes::managed_state::ManagedState;
use chrono::Utc;
use systemprompt::analytics::snapshots::SnapshotRangeJob;
use systemprompt::identifiers::InventoryEntryId;

pub(super) async fn collect_rows(
    managed: &ManagedState,
    plan: &Plan,
    evidence: &mut Evidence<'_>,
    now: chrono::DateTime<Utc>,
) -> AdminResult<(
    Vec<PortfolioResourceRow>,
    usize,
    Option<InventoryEntryId>,
    Vec<SnapshotRangeJob>,
)> {
    let mut items = managed
        .portfolio
        .list(
            &managed.owner,
            &PortfolioFilter {
                after: plan.after.as_ref(),
                kind: Some(plan.inventory_kind),
                search: &plan.search,
                window: plan.range.window,
                limit: 51,
            },
        )
        .await?;
    let more = items.len() > 50;
    items.truncate(50);
    let next = more
        .then(|| items.last().map(|i| i.entry.entry_id.clone()))
        .flatten();
    let mut keys = std::collections::HashSet::new();
    for item in &items {
        if keys.insert((item.entry.kind.clone(), item.entry.resource_key.clone())) {
            evidence.managed.extend(
                managed
                    .repository
                    .list_resources_for_key(
                        &managed.owner,
                        &item.entry.kind,
                        &item.entry.resource_key,
                        51,
                    )
                    .await?,
            );
        }
    }
    let window = (plan.range.from.to_string(), plan.range.to.to_string());
    let mut pending_jobs = 0;
    let mut jobs = Vec::new();
    let mut rows: Vec<PortfolioResourceRow> = Vec::new();
    for mut item in items {
        if plan.range.custom
            && plan.filter_group.is_empty()
            && let Some(resource) = item.entry.resource_id.as_ref()
        {
            let job = range::custom_job(managed, Some(resource), &plan.range, now).await?;
            item.snapshot = job.result.clone();
            if matches!(job.state.as_str(), "pending" | "leased") {
                pending_jobs += 1;
            }
            jobs.push(job);
        }
        if !plan.filter_group.is_empty() {
            item.snapshot = None;
        }
        rows.extend(expand_rows(item, evidence, (&window.0, &window.1)));
    }
    if !plan.filter_marketplace.is_empty() {
        rows.retain(|row| {
            row.marketplaces
                .iter()
                .any(|m| m.id.as_str() == plan.filter_marketplace)
        });
    }
    Ok((rows, pending_jobs, next, jobs))
}

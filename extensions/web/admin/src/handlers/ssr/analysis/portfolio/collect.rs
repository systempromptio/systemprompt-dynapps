//! Turning the inventory into rows: every entry the plan admits, its
//! evidence attached, with custom ranges scheduled rather than read.

use chrono::Utc;
use systemprompt::analytics::snapshots::SnapshotRangeJob;

use super::evidence::{Evidence, expand_rows};
use super::plan::{Plan, list_inventory};
use super::range;
use super::rows::InventoryRow;
use crate::error::AdminResult;
use crate::routes::managed_state::ManagedState;

pub(crate) struct Collected {
    pub(super) rows: Vec<InventoryRow>,
    pub(super) pending_jobs: usize,
    pub(super) range_jobs: Vec<SnapshotRangeJob>,
}

pub(super) async fn collect_rows(
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
    Ok(collected)
}

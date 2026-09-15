//! Render one bounded page without reinterpreting snapshot metrics or current
//! access as history.
use super::{Plan, PortfolioPage, SkillRow, groups};
use crate::repositories::analysis::inventory_index::MarketplaceRef;
use crate::types::groups::GroupSummary;
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotHealth, SnapshotRangeJob};
use systemprompt::identifiers::InventoryEntryId;
use systemprompt::marketplace::inventory::{InstallationCoverageStatus, InventoryStatus};

pub(super) struct LoadedPage {
    pub(super) plan: Plan,
    pub(super) rows: Vec<SkillRow>,
    pub(super) pending_jobs: usize,
    pub(super) next_cursor: Option<InventoryEntryId>,
    pub(super) range_jobs: Vec<SnapshotRangeJob>,
    pub(super) totals: Option<FeedbackSnapshot>,
    pub(super) health: SnapshotHealth,
    pub(super) inventory: InventoryStatus,
    pub(super) installations: InstallationCoverageStatus,
    pub(super) user_groups: Vec<GroupSummary>,
}

pub(super) fn assemble(input: LoadedPage) -> PortfolioPage {
    let LoadedPage {
        plan,
        rows,
        pending_jobs,
        next_cursor,
        range_jobs,
        totals,
        health,
        inventory,
        installations,
        user_groups,
    } = input;
    let unbound = rows.iter().filter(|r| r.usage_status == "unbound").count();
    let conflicting = rows
        .iter()
        .filter(|r| r.usage_status == "conflicting")
        .count();
    let kpis = groups::kpis(&rows, totals.as_ref());
    let groups = groups::group_rows(rows, |_| None);
    let next_href = next_cursor.as_ref().map(|cursor| format!(
        "/admin/analysis/resources?after={}&kind={}&start={}&end={}&marketplace={}&group={}&search={}",
        urlencoding::encode(cursor.as_str()), plan.kind, plan.range.from, plan.range.to,
        urlencoding::encode(&plan.filter_marketplace), urlencoding::encode(&plan.filter_group), urlencoding::encode(&plan.search)
    ));
    let unknown_usage = totals
        .as_ref()
        .filter(|snapshot| snapshot.suppressed_days == 0)
        .map(|snapshot| snapshot.metrics.invocations - snapshot.metrics.verified_invocations);
    PortfolioPage {
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
        user_groups,
        filter_marketplace: plan.filter_marketplace,
        filter_group: plan.filter_group,
        start: plan.range.from.to_string(),
        end: plan.range.to.to_string(),
        search: plan.search,
        days: plan.range.window,
        generated_at: health.generated_at.map_or_else(
            || "Awaiting first snapshot".to_owned(),
            |at| at.to_rfc3339(),
        ),
        totals,
        health,
        inventory,
        installations,
        pending_jobs,
        range_jobs,
        unknown_usage,
        unbound,
        conflicting,
        operation_key: uuid::Uuid::new_v4().to_string(),
        next_cursor,
        next_href,
        metrics_notice: "Rows show resource-wide retained usage, repeated under current catalog placement. Historical plugin, marketplace and user-group metrics are unavailable. Organization totals are shown only without filters. Inventory counts describe this page. Missing or suppressed evidence is unknown; shared request costs must not be summed across resources.".to_owned(),
    }
}

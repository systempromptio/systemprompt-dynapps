//! Presentation preserves unavailable measurements and separate receipt
//! coverage.

use crate::error::AdminResult;
use crate::repositories::analysis::portfolio_snapshots::PortfolioEntry;
use serde::Serialize;
use systemprompt::analytics::snapshots::FeedbackSnapshot;
use systemprompt::marketplace::inventory::{InstallationCoverage, InventoryEntry};

#[derive(Serialize)]
pub(super) struct ResourceRow {
    pub(super) entry: InventoryEntry,
    pub(super) snapshot: Option<FeedbackSnapshot>,
    pub(super) installation: Option<InstallationCoverage>,
    pub(super) href: Option<String>,
    pub(super) cost: Option<String>,
    pub(super) p95: Option<String>,
    pub(super) users: String,
    pub(super) usage_status: &'static str,
}
impl ResourceRow {
    pub(super) fn from_entry(item: PortfolioEntry) -> AdminResult<Self> {
        let cost = item
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.metrics.priced_requests > 0)
            .map(|snapshot| {
                snapshot
                    .spend_by_currency
                    .iter()
                    .map(|(currency, micros)| {
                        format!(
                            "{currency} {}.{:06}",
                            micros / 1_000_000,
                            micros % 1_000_000
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            });
        let p95 = item
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.histogram.percentile_upper_bound_micros(95))
            .transpose()?
            .flatten()
            .map(|micros| format!("{}.{:03}", micros / 1000, micros % 1000));
        let users = item
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.distinct_users)
            .map_or_else(|| "Unavailable".to_owned(), |count| count.to_string());
        let href = match item.entry.kind.as_str() {
            "skill" | "plugin" => Some(format!(
                "/admin/catalog/{}s/{}",
                item.entry.kind,
                urlencoding::encode(&item.entry.resource_key)
            )),
            _ => None,
        };
        let usage_status = if item.snapshot.is_some() {
            "ready"
        } else if item.entry.resource_id.is_some() {
            "pending"
        } else {
            "unbound"
        };
        Ok(Self {
            entry: item.entry,
            snapshot: item.snapshot,
            installation: item.installation,
            href,
            cost,
            p95,
            users,
            usage_status,
        })
    }
}

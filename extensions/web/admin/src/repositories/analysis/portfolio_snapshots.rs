//! Bounded portfolio pages join inventory and retained aggregate/installation
//! snapshots.

use crate::error::{AdminError, AdminResult};
use sqlx::PgPool;
use systemprompt::analytics::snapshots::FeedbackSnapshot;
use systemprompt::identifiers::{InventoryEntryId, UserId};
use systemprompt::marketplace::inventory::{InstallationCoverage, InventoryEntry};

/// One canonical inventory entry with independent usage and installation
/// coverage.
#[derive(Debug, serde::Serialize)]
pub struct PortfolioEntry {
    pub entry: InventoryEntry,
    pub snapshot: Option<FeedbackSnapshot>,
    pub installation: Option<InstallationCoverage>,
}
/// Snapshot-backed administrative portfolio query repository.
#[derive(Debug, Clone)]
pub struct PortfolioSnapshotsRepository {
    pool: PgPool,
}
/// A bounded inventory filter with stable canonical identity pagination.
#[derive(Debug)]
pub struct PortfolioFilter<'a> {
    pub after: Option<&'a InventoryEntryId>,
    pub kind: Option<&'a str>,
    pub search: &'a str,
    pub window: u32,
    pub limit: u32,
}
impl PortfolioSnapshotsRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    pub async fn list(
        &self,
        owner: &UserId,
        filter: &PortfolioFilter<'_>,
    ) -> AdminResult<Vec<PortfolioEntry>> {
        if !(1..=100).contains(&filter.limit)
            || ![1, 7, 30, 90, 365].contains(&filter.window)
            || filter.search.len() > 200
        {
            return Err(AdminError::BadRequest("Invalid portfolio page".to_owned()));
        }
        let after = filter.after.map(InventoryEntryId::as_str);
        let window = i32::try_from(filter.window)
            .map_err(|error| AdminError::BadRequest(error.to_string()))?;
        let limit = i64::from(filter.limit);
        let rows=sqlx::query!(r#"SELECT m.record,s.body AS "snapshot?",c.body AS "installation?"
   FROM managed_inventory_entries e JOIN managed_inventory_membership m ON m.owner_id=e.owner_id AND m.entry_id=e.entry_id AND m.effective_until IS NULL
   LEFT JOIN analytics_feedback_snapshots s ON s.owner_id=e.owner_id AND s.scope=e.resource_id AND s.window_days=$4
   LEFT JOIN managed_installation_coverage c ON c.owner_id=e.owner_id AND c.resource_id=e.resource_id
   WHERE e.owner_id=$1 AND ($2::text IS NULL OR e.entry_id>$2) AND ($3::text IS NULL OR e.kind=$3)
    AND ($5='' OR strpos(lower(e.resource_key),lower($5))>0)
   ORDER BY e.entry_id LIMIT $6"#,owner.as_str(),after,filter.kind,window,filter.search,limit).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|row| {
                Ok(PortfolioEntry {
                    entry: serde_json::from_value(row.record)?,
                    snapshot: row.snapshot.map(serde_json::from_value).transpose()?,
                    installation: row.installation.map(serde_json::from_value).transpose()?,
                })
            })
            .collect()
    }
}

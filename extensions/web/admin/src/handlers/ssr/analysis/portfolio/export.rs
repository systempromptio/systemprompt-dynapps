//! The resource inventory as the export surface reads it.

use chrono::Utc;
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;

use super::collect::collect_rows;
use super::evidence::Evidence;
use super::plan::{Plan, plan};
use super::rows::InventoryRow;
use super::{PortfolioQuery, narrow_to_marketplace};
use crate::error::AdminError;
use crate::export::ExportView;

// Why: the export is the inventory the page lists, before it is grouped by
// marketplace — one row per resource, narrowed to the picked marketplace.
pub(crate) async fn export_rows(
    managed: &ManagedState,
    pool: &PgPool,
    user: &UserContext,
    query: PortfolioQuery,
) -> AdminResult<Vec<InventoryRow>> {
    require_admin(user)?;
    let now = Utc::now();
    let plan = plan(query, now)?;
    let evidence = Evidence::gather(managed, pool, &plan.index, plan.range.retained()).await?;
    let mut collected = collect_rows(managed, &plan, &evidence, now).await?;
    narrow_to_marketplace(&mut collected, &plan);
    Ok(collected.rows)
}

pub(super) fn require_admin(user: &UserContext) -> AdminResult<()> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for organizational resource analysis".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn export_view(plan: &Plan) -> ExportView {
    ExportView::single(
        "analysis-resources",
        &crate::export::view::query_string(&[
            ("days", Some(&plan.range.window.to_string())),
            ("kind", Some(&plan.kind)),
            ("search", Some(&plan.search)),
            ("marketplace", Some(&plan.filter_marketplace)),
        ]),
    )
}

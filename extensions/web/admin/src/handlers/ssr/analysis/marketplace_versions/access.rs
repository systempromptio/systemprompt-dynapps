//! Who may read which marketplace's version history.

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::repositories::analysis::marketplace_versions::{
    MarketplaceVersionMetricsRow, VersionWindow, list_marketplace_version_metrics,
};
use crate::types::UserContext;

pub(crate) fn require_versions_reader(user: &UserContext) -> AdminResult<()> {
    if !user.has_scoped_console() {
        return Err(AdminError::Forbidden(
            "Marketplace participation required for version history".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn may_read_marketplace(user: &UserContext, marketplace_id: &MarketplaceId) -> bool {
    user.is_console
        || user
            .participation
            .includes_marketplace(marketplace_id.as_str())
}

pub(super) async fn readable_versions(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
    user: &UserContext,
) -> AdminHtmlResult<Vec<MarketplaceVersionMetricsRow>> {
    let rows = if may_read_marketplace(user, marketplace_id) {
        list_marketplace_version_metrics(pool, window, marketplace_id).await?
    } else {
        Vec::new()
    };
    if rows.is_empty() {
        return Err(AdminError::NotFound(format!(
            "No version of marketplace '{marketplace_id}' has been recorded"
        ))
        .into());
    }
    Ok(rows)
}

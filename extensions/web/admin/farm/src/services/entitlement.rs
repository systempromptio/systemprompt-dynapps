//! Who may launch and read a farm's runs.
//!
//! A console user and the farm's owner always may. Anyone else must hold the
//! farm's marketplace: the same `marketplace/<id>` grant that decides which
//! marketplaces a user sees in the console, resolved through
//! `systemprompt_web_admin_core::repositories::entitlement::resolve_user_matrix`
//! exactly as `scope::participation` resolves marketplace participation, so a
//! farm is usable by precisely the people its marketplace is visible to.

use std::collections::BTreeSet;

use sqlx::PgPool;
use systemprompt_web_admin_core::repositories::entitlement::{SectionInput, resolve_user_matrix};
use systemprompt_web_admin_core::types::UserContext;

use crate::error::{FarmError, FarmResult};
use crate::repositories::rows::{FarmRow, RunRow};

const MARKETPLACE_ENTITY: &str = "marketplace";
const ALLOW: &str = "allow";

#[must_use]
pub fn is_farm_owner(ctx: &UserContext, farm: &FarmRow) -> bool {
    farm.owner_user_id.as_deref() == Some(ctx.user_id.as_str())
}

pub async fn list_entitled_marketplaces(
    pool: &PgPool,
    ctx: &UserContext,
    marketplace_ids: &BTreeSet<String>,
) -> FarmResult<BTreeSet<String>> {
    if marketplace_ids.is_empty() {
        return Ok(BTreeSet::new());
    }
    let section: SectionInput = (
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        marketplace_ids
            .iter()
            .map(|id| (id.clone(), id.clone(), None))
            .collect(),
    );
    let Some(matrix) = resolve_user_matrix(pool, &ctx.user_id, vec![section]).await? else {
        return Ok(BTreeSet::new());
    };
    Ok(matrix
        .sections
        .into_iter()
        .flat_map(|s| s.rows)
        .filter(|row| row.effective == ALLOW)
        .map(|row| row.entity_id)
        .collect())
}

pub async fn may_use_farm(pool: &PgPool, ctx: &UserContext, farm: &FarmRow) -> FarmResult<bool> {
    if ctx.is_console || is_farm_owner(ctx, farm) {
        return Ok(true);
    }
    let wanted = BTreeSet::from([farm.marketplace_id.as_str().to_owned()]);
    Ok(!list_entitled_marketplaces(pool, ctx, &wanted)
        .await?
        .is_empty())
}

pub async fn require_farm_access(
    pool: &PgPool,
    ctx: &UserContext,
    farm: &FarmRow,
) -> FarmResult<()> {
    if may_use_farm(pool, ctx, farm).await? {
        return Ok(());
    }
    Err(FarmError::Forbidden(format!(
        "farm {} requires the marketplace {}",
        farm.id.as_str(),
        farm.marketplace_id.as_str()
    )))
}

#[must_use]
pub fn may_view_run(ctx: &UserContext, run: &RunRow, farm: Option<&FarmRow>) -> bool {
    ctx.is_console || run.user_id == ctx.user_id || farm.is_some_and(|f| is_farm_owner(ctx, f))
}

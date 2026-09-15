//! Explicit binding of a configured inventory entry to the managed resource
//! that shares its name, so retained snapshots and coverage can attach to it.

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::Extension;
use axum::extract::{Form, Path};
use axum::http::HeaderMap;
use axum::response::Redirect;
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::identifiers::{InventoryEntryId, ManagedResourceId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BindForm {
    resource_id: ManagedResourceId,
}

pub(crate) async fn bind(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(entry): Path<InventoryEntryId>,
    headers: HeaderMap,
    Form(form): Form<BindForm>,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    state
        .repository
        .bind_inventory_resource(&state.owner, &user.user_id, &entry, &form.resource_id)
        .await?;
    let root = crate::handlers::shared::get_services_path()?;
    let services = systemprompt::loader::ConfigLoader::load().map_err(AdminError::internal)?;
    systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone())
        .refresh(&state.owner, &root, &services)
        .await?;
    Ok(Redirect::to("/admin/analysis/resources"))
}

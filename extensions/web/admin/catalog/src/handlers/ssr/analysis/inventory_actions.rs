//! Explicit binding of a configured inventory entry to the managed resource
//! that shares its name, so retained snapshots and coverage can attach to it,
//! and the on-demand inventory sync that publishes every configured skill.

use axum::Extension;
use axum::extract::rejection::ExtensionRejection;
use axum::extract::{Form, FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::response::Redirect;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{InventoryEntryId, ManagedResourceId};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::{ExtensionBoot, WriteRequest};
use systemprompt_web_admin_core::extension_services::ExtensionServices;
use systemprompt_web_admin_core::state::ManagedState;
use systemprompt_web_admin_core::types::UserContext;

pub(crate) struct InventoryDeps {
    services: ExtensionServices,
    state: Arc<ManagedState>,
}

impl<S: Send + Sync> FromRequestParts<S> for InventoryDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(services) = Extension::from_request_parts(parts, state).await?;
        let Extension(managed) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            services,
            state: managed,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BindForm {
    resource_id: ManagedResourceId,
}

pub(crate) async fn bind(
    InventoryDeps { services, state }: InventoryDeps,
    write: WriteRequest,
    Extension(user): Extension<UserContext>,
    Path(entry): Path<InventoryEntryId>,
    Form(form): Form<BindForm>,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    write.require_origin()?;
    state
        .repository
        .bind_inventory_resource(&state.owner, &user.user_id, &entry, &form.resource_id)
        .await?;
    let root = systemprompt_web_admin_core::request::get_services_path(&write.boot);
    let services = services.config();
    systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone())
        .refresh(&state.owner, &root, &services, sources_context(&write.boot))
        .await?;
    Ok(Redirect::to(SKILLS_PAGE))
}

pub(crate) async fn sync_inventory(
    InventoryDeps { services, state }: InventoryDeps,
    write: WriteRequest,
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    write.require_origin()?;
    let root = systemprompt_web_admin_core::request::get_services_path(&write.boot);
    let services = services.config();
    let service =
        systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone());
    service
        .refresh(&state.owner, &root, &services, sources_context(&write.boot))
        .await?;
    service
        .publish_latest(
            &systemprompt::marketplace::inventory::BaselineScope {
                owner: &state.owner,
                actor: &user.user_id,
                root: &root,
                services: &services,
                sources: sources_context(&write.boot),
            },
            &mut systemprompt::marketplace::inventory::PublishGuard::default(),
        )
        .await?;
    systemprompt_web_admin_core::repositories::sync::sources_db::record_service_sources(
        &pool,
        &write.boot,
        &services,
    )
    .await
    .map_err(AdminError::internal)?;
    Ok(Redirect::to(SKILLS_PAGE))
}

const SKILLS_PAGE: &str = systemprompt_web_admin_core::ssr::analysis_urls::ANALYSIS_SKILLS_URL;

fn sources_context(
    boot: &ExtensionBoot,
) -> systemprompt::marketplace::inventory::SourcesContext<'_> {
    systemprompt::marketplace::inventory::SourcesContext {
        profile: boot.profile(),
        active_root: Some(boot.active_services_root()),
    }
}

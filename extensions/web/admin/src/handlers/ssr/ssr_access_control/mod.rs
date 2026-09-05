//! `/admin/access-control` — the unified access-control editor.
//!
//! One page for the whole grant model: pick a group or a project to write a
//! band rule that covers everyone in it, or pick a person to see the resolved
//! matrix and write a per-user override. Reading is the CONSOLE tier so a
//! project manager can audit; every control that writes is gated on MANAGE.

mod data;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, Role, UserContext};

use view::{AcStatsView, AccessControlPageData};

pub(crate) async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let services_path = shared::get_services_path()?;

    let users = data::load_users(&pool).await;
    let groups = data::load_groups(&pool, &users).await;
    let projects = data::load_projects(&pool).await;

    let stats = AcStatsView {
        groups: groups.len(),
        projects: projects.len(),
        users: users.len(),
    };

    let page = AccessControlPageData {
        page: "access-control",
        title: "Access control",
        can_write: user_ctx.is_admin,
        group_ids: groups.iter().map(|g| g.id.clone()).collect(),
        project_ids: projects.iter().map(|p| p.id.clone()).collect(),
        entity_catalogue: data::entity_catalogue(&services_path),
        known_roles: Role::ALL.iter().map(|r| r.as_str().to_owned()).collect(),
        stats,
        groups,
        projects,
    };

    Ok(super::render_typed_page(
        &engine,
        "access-control",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

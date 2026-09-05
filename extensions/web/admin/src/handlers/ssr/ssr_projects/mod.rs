//! `/admin/projects` and `/admin/projects/{project_id}`.
//!
//! A project is work attribution and an ACL subject; a group is people and
//! entitlement. The two pages look alike on purpose, but the project detail
//! has no tabs: a project has no membership sources to reconcile beyond the
//! same directory-versus-manual split, and everything it shows fits one view.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::types::{BreadcrumbView, ProjectsPageData};

mod data;
mod view;

pub(crate) async fn projects_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let projects = data::load_project_rows(&pool).await;
    let data = ProjectsPageData {
        page: "projects",
        title: "Projects",
        stats: view::project_stats(&projects),
        projects,
        can_manage: user_ctx.is_admin,
    };

    Ok(super::render_typed_page(
        &engine, "projects", &data, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) async fn project_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(project) = repositories::projects::crud::find_project(&pool, &project_id).await?
    else {
        return Err(AdminError::NotFound("No such project.".to_owned()).into());
    };

    let loaded = data::load_project_detail(&pool, &project_id, &user_ctx).await;
    let data = view::detail_page(project, loaded, &user_ctx);

    Ok(super::render_typed_page(
        &engine,
        "project-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(super) fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Projects", "/admin/projects"),
        BreadcrumbView::current(name),
    ]
}

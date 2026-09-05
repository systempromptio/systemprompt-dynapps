//! Read-only catalog admin pages for the three installable entity families:
//!
//! - `/admin/catalog/plugins` — plugins (collections) from
//!   `services/plugins/*/config.yaml`, each referencing skills, MCP servers,
//!   agents, and hooks.
//! - `/admin/catalog/skills` — skills from `services/skills/*`.
//! - `/admin/catalog/mcp` — MCP servers from `services/mcp/*`.
//!
//! Each family has a list page and a detail page. Detail pages surface the
//! plugin ↔ member relationship in both directions. All pages are strictly
//! read-only: operators edit `services/*.yaml` and restart.

mod data;
pub(crate) mod marketplaces;
mod view;
mod view_models;
mod visibility;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{
    ENTITY_MCP_SERVER, ENTITY_PLUGIN, ENTITY_SKILL, MarketplaceContext, UserContext,
};

use super::ssr::ssr_helpers::render_typed_page;
use view::assignment_counts_by_type;
use view_models::VisibilityInput;

// Why: two cheap reads the badge needs on every catalog page. Both are
// best-effort: a badge that cannot be computed renders as restricted, which is
// the safe reading, rather than failing the page.
async fn visibility_inputs(
    pool: &PgPool,
    services_path: &std::path::Path,
) -> (
    Vec<repositories::marketplace::manifests::MarketplaceConfigSummary>,
    Vec<crate::types::access_control::AccessControlRule>,
) {
    let manifests = repositories::marketplace::manifests::list_marketplace_configs(services_path)
        .unwrap_or_default();
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "catalog: rule listing failed"))
        .unwrap_or_default();
    (manifests, rules)
}

pub(crate) async fn plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_PLUGIN).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let plugins = view_models::plugin_rows(
        catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let page = view::PluginsPageData {
        page: "plugins",
        title: "Plugins",
        plugins_count: plugins.len(),
        plugins,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-plugins",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn plugin_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_PLUGIN).await;
    let assignment_count = counts.get(&plugin_id).copied().unwrap_or(0);
    let page = view_models::plugin_detail(&catalog, &plugin_id, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such plugin.".to_owned()))?;
    Ok(render_typed_page(
        &engine,
        "catalog-plugin-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn skills_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let skills = view_models::skill_rows(
        &catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let page = view::SkillsPageData {
        page: "skills",
        title: "Skills",
        skills_count: skills.len(),
        skills,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-skills",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn skill_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(skill_id): Path<String>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let assignment_count = counts.get(&skill_id).copied().unwrap_or(0);
    let skill = systemprompt::identifiers::SkillId::new(&skill_id);
    let page = view_models::skill_detail(&catalog, &skill, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such skill.".to_owned()))?;
    Ok(render_typed_page(
        &engine,
        "catalog-skill-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let servers = view_models::mcp_rows(&catalog, &counts);
    let page = view::McpPageData {
        page: "mcp",
        title: "MCP servers",
        servers_count: servers.len(),
        servers,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-mcp",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn mcp_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(mcp_id): Path<String>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_MCP_SERVER).await;
    let assignment_count = counts.get(&mcp_id).copied().unwrap_or(0);
    let page = view_models::mcp_detail(&catalog, &mcp_id, assignment_count)
        .ok_or_else(|| AdminError::NotFound("No such MCP server.".to_owned()))?;
    Ok(render_typed_page(
        &engine,
        "catalog-mcp-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn admin_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

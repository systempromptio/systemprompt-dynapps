//! `/admin/catalog/marketplaces` — the marketplace list with its audience
//! matrix, and the per-marketplace detail page.
//!
//! A marketplace is the unit entitlement is granted on, so these two pages
//! answer a question the plugin and skill pages cannot: not "what is in the
//! catalog" but "who reaches it". Both pages are read-only — the manifests
//! live in `services/marketplaces/*/config.yaml` and rules are edited on the
//! access-control page.

mod data;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, Role, UserContext};

use self::view::{
    CrumbView, MarketplaceCardView, MarketplaceDetailData, MarketplacesPageData, MemberLinkView,
    marketplace_url,
};
use super::super::ssr::ssr_helpers::render_typed_page;
use super::view::{mcp_url, plugin_url, skill_url};

fn known_roles() -> Vec<String> {
    Role::ALL.iter().map(|r| r.as_str().to_owned()).collect()
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

pub(crate) async fn marketplaces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;
    let manifests = data::load_manifests(&path);
    let audience = data::audience_matrix(&pool, &manifests, &known_roles()).await;

    let marketplaces: Vec<MarketplaceCardView> = manifests
        .iter()
        .map(|m| MarketplaceCardView {
            id: m.id.clone(),
            name: m.name.clone(),
            description: m.description.clone(),
            version: m.version.clone(),
            enabled: m.enabled,
            visibility: m.visibility.clone(),
            detail_url: marketplace_url(&m.id),
            roles: m.access.roles.clone(),
            groups: m.access.groups.clone(),
            projects: m.access.projects.clone(),
            plugin_count: m.plugins.len(),
            mcp_count: m.mcp_servers.len(),
            default_included: m.access.default_included,
        })
        .collect();

    let page = MarketplacesPageData {
        page: "marketplaces",
        title: "Marketplaces",
        marketplaces_count: marketplaces.len(),
        marketplaces,
        audience,
    };
    Ok(render_typed_page(
        &engine,
        "catalog-marketplaces",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn member_links(ids: &[String], url: fn(&str) -> String) -> Vec<MemberLinkView> {
    ids.iter()
        .map(|id| MemberLinkView {
            id: id.clone(),
            name: id.clone(),
            url: url(id),
        })
        .collect()
}

// Why: a marketplace names plugins, and skills follow the plugins that ship
// them — there is no marketplace-level skill list to read, so the skills shown
// here are derived from the member plugins rather than declared.
fn skills_of(services_path: &std::path::Path, plugin_ids: &[String]) -> Vec<MemberLinkView> {
    let catalog = crate::repositories::marketplace::plugins::list_plugin_catalog(services_path)
        .unwrap_or_default();
    let mut out: Vec<MemberLinkView> = Vec::new();
    for plugin in catalog.iter().filter(|p| plugin_ids.contains(&p.id)) {
        for skill in &plugin.skills {
            let id = skill.as_str().to_owned();
            if out.iter().any(|s| s.id == id) {
                continue;
            }
            out.push(MemberLinkView {
                url: skill_url(&id),
                name: id.clone(),
                id,
            });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub(crate) async fn marketplace_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(marketplace_id): Path<String>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;
    let manifests = data::load_manifests(&path);
    let manifest = manifests
        .iter()
        .find(|m| m.id == marketplace_id)
        .cloned()
        .ok_or_else(|| AdminError::NotFound("No such marketplace.".to_owned()))?;

    let (group_audience, role_audience) =
        data::audience_for(&pool, &manifests, &marketplace_id, &known_roles()).await;

    let plugins = member_links(&manifest.plugins, plugin_url);
    let mcp_servers = member_links(&manifest.mcp_servers, mcp_url);
    let skills = skills_of(&path, &manifest.plugins);

    let page = MarketplaceDetailData {
        page: "marketplace-detail",
        title: manifest.name.clone(),
        breadcrumbs: vec![
            CrumbView {
                label: "Marketplaces".to_owned(),
                href: Some("/admin/catalog/marketplaces"),
            },
            CrumbView {
                label: manifest.name.clone(),
                href: None,
            },
        ],
        id: manifest.id,
        name: manifest.name,
        description: manifest.description,
        version: manifest.version,
        enabled: manifest.enabled,
        visibility: manifest.visibility,
        default_included: manifest.access.default_included,
        justification: manifest.access.justification,
        source_path: manifest.source_path,
        roles: manifest.access.roles,
        groups: manifest.access.groups,
        projects: manifest.access.projects,
        plugins_count: plugins.len(),
        skills_count: skills.len(),
        mcp_count: mcp_servers.len(),
        plugins,
        skills,
        mcp_servers,
        group_audience,
        role_audience,
        access_control_url: "/admin/access/control",
    };
    Ok(render_typed_page(
        &engine,
        "catalog-marketplace-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

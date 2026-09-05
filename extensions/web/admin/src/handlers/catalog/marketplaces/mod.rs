//! `/admin/marketplaces` — the marketplace list with its audience
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
    MarketplaceCardView, MarketplaceDetailData, MarketplaceKpiView, MarketplacesPageData,
    MemberLinkView, marketplace_url,
};
use super::super::ssr::ssr_helpers::render_typed_page;
use super::view::{mcp_url, plugin_url, skill_url};
use crate::handlers::ssr::types::BreadcrumbView;

// Why: the six facts an operator scans before opening any one marketplace.
// "Reachable" is the resolved count, so a manifest that grants a group nothing
// can reach reads as zero here rather than as its declared list.
fn list_kpis(
    cards: &[MarketplaceCardView],
    audience: &view::AudienceMatrixView,
) -> Vec<MarketplaceKpiView> {
    let enabled = cards.iter().filter(|c| c.enabled).count();
    let open = cards.iter().filter(|c| c.default_included).count();
    let plugins: usize = cards.iter().map(|c| c.plugin_count).sum();
    let skills: usize = cards.iter().map(|c| c.skill_count).sum();
    let assigned: usize = cards.iter().map(|c| c.assigned_group_count).sum();
    let unreachable = cards.iter().filter(|c| c.allowed_subjects == 0).count();
    let kpi =
        |label: &'static str, value: String, sub: String, tone: &'static str| MarketplaceKpiView {
            label,
            value,
            sub,
            tone,
        };
    vec![
        kpi(
            "Marketplaces",
            cards.len().to_string(),
            format!("{enabled} enabled"),
            "",
        ),
        kpi(
            "Group grants",
            assigned.to_string(),
            format!("across {} subjects", audience.rows.len()),
            "",
        ),
        kpi(
            "Reachable by nobody",
            unreachable.to_string(),
            "no group or role resolves to allow".to_owned(),
            if unreachable > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "Open to the estate",
            open.to_string(),
            "default_included is true".to_owned(),
            if open > 0 { "warn" } else { "ok" },
        ),
        kpi(
            "Plugins",
            plugins.to_string(),
            "declared members".to_owned(),
            "",
        ),
        kpi(
            "Skills",
            skills.to_string(),
            "carried by those plugins".to_owned(),
            "",
        ),
    ]
}

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
    axum::extract::Query(query): axum::extract::Query<super::CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;
    let path = shared::get_services_path()?;
    let manifests = data::load_manifests(&path);
    let audience = data::audience_matrix(&pool, &manifests, &known_roles()).await;
    let grants = data::group_grants(&pool).await;
    let plugin_catalog =
        crate::repositories::marketplace::plugins::list_plugin_catalog(&path).unwrap_or_default();

    let marketplaces: Vec<MarketplaceCardView> = manifests
        .iter()
        .map(|m| {
            let assigned_groups = grants.get(&m.id).cloned().unwrap_or_default();
            // Why: the resolved count, not the declared one. A group listed in
            // the manifest that a deny rule closes is not an audience, and the
            // two numbers side by side are how that shows up.
            let allowed_subjects = audience
                .rows
                .iter()
                .filter(|row| {
                    row.cells
                        .iter()
                        .any(|c| c.marketplace_id == m.id && c.is_allow)
                })
                .count();
            MarketplaceCardView {
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
                skill_count: skills_of(&plugin_catalog, &m.plugins).len(),
                mcp_count: m.mcp_servers.len(),
                default_included: m.access.default_included,
                assigned_group_count: assigned_groups.len(),
                assigned_groups,
                allowed_subjects,
            }
        })
        .collect();

    let search = query.q.unwrap_or_default();
    let kpis = list_kpis(&marketplaces, &audience);
    let marketplaces: Vec<MarketplaceCardView> = marketplaces
        .into_iter()
        .filter(|m| super::sorting::matches(&[&m.id, &m.name, &m.description], &search))
        .collect();

    let page = MarketplacesPageData {
        page: "marketplaces",
        title: "Marketplaces",
        subtitle: "The unit entitlement is granted on. Each one names plugins and MCP servers, and each group holds a set of them.",
        breadcrumbs: vec![BreadcrumbView::current("Marketplaces")],
        // Why: the tiles count every marketplace, not the filtered view — a
        // total that moved with the search box would mean two different things.
        kpis,
        marketplaces_count: marketplaces.len(),
        marketplaces,
        audience,
        access_control_url: "/admin/access-control?entity_type=marketplace",
        search,
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
// here are derived from the member plugins rather than declared. The catalog
// is passed in because the list page needs this once per marketplace and
// re-reading the plugin tree each time made one page render walk it four times.
fn skills_of(catalog: &[crate::types::PluginDetail], plugin_ids: &[String]) -> Vec<MemberLinkView> {
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
    let group_assignments = data::group_assignments(&pool, &marketplace_id, &group_audience).await;

    let plugins = member_links(&manifest.plugins, plugin_url);
    let mcp_servers = member_links(&manifest.mcp_servers, mcp_url);
    let plugin_catalog =
        crate::repositories::marketplace::plugins::list_plugin_catalog(&path).unwrap_or_default();
    let skills = skills_of(&plugin_catalog, &manifest.plugins);

    let page = MarketplaceDetailData {
        page: "marketplace-detail",
        title: manifest.name.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Marketplaces", "/admin/marketplaces"),
            BreadcrumbView::current(manifest.name.clone()),
        ],
        id: manifest.id,
        name: manifest.name,
        description: manifest.description,
        version: manifest.version,
        enabled: manifest.enabled,
        visibility: manifest.visibility,
        default_included: manifest.access.default_included,
        default_included_label: if manifest.access.default_included {
            "Yes"
        } else {
            "No"
        },
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
        assigned_count: group_assignments.iter().filter(|g| g.assigned).count(),
        group_assignments_count: group_assignments.len(),
        group_assignments,
        access_control_url: "/admin/access-control",
    };
    Ok(render_typed_page(
        &engine,
        "catalog-marketplace-detail",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}

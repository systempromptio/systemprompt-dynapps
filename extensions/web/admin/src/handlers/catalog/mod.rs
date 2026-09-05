//! Read-only catalog admin pages for the three installable entity families:
//!
//! - `/admin/plugins` — plugins (collections) from
//!   `services/plugins/*/config.yaml`, each referencing skills, MCP servers,
//!   agents, and hooks.
//! - `/admin/skills` — skills from `services/skills/*`.
//! - `/admin/mcp` — MCP servers from `services/mcp/*`, whose pages live in
//!   [`mcp`] because they also read the runtime tables.
//!
//! Each family has a list page and a detail page. Detail pages surface the
//! plugin ↔ member relationship in both directions. The plugin and skill pages
//! are strictly read-only: operators edit `services/*.yaml` and restart.

mod data;
pub(crate) mod marketplaces;
pub(crate) mod mcp;
pub(crate) mod sorting;
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
use crate::types::{ENTITY_PLUGIN, ENTITY_SKILL, MarketplaceContext, UserContext};

use super::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use sorting::{SortColumn, apply_direction, direction, matches, preserved_search, sort_headers};
use view::{CatalogKpiView, assignment_counts_by_type};
use view_models::VisibilityInput;

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct CatalogListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    // Why: the filter is a server round trip rather than a class toggle, so the
    // row count, the empty state and the URL all agree — a client-side hide
    // leaves the filtered rows in the DOM and the count above them wrong.
    pub q: Option<String>,
}

fn kpi(label: &'static str, value: String, sub: String, tone: &'static str) -> CatalogKpiView {
    CatalogKpiView {
        label,
        value,
        sub,
        tone,
    }
}

// Why: the entry columns two of the three catalog lists share. Sorting is on
// the assembled rows rather than in a query because the catalog is read from
// YAML on disk — there is no query to push an ORDER BY into.
fn entry_columns(members_label: &'static str, members_hint: &'static str) -> Vec<SortColumn> {
    vec![
        SortColumn {
            key: "name",
            label: "Name",
            class: "",
            hint: "The id as declared under services/",
        },
        SortColumn {
            key: "status",
            label: "Status",
            class: "",
            hint: "Whether the entry is enabled in its YAML",
        },
        SortColumn {
            key: "members",
            label: members_label,
            class: "sp-table__cell--num",
            hint: members_hint,
        },
        SortColumn {
            key: "visibility",
            label: "Visibility",
            class: "",
            hint: "Public, or restricted to the roles and groups a rule names",
        },
        SortColumn {
            key: "grants",
            label: "Grants",
            class: "sp-table__cell--num",
            hint: "Access-control rules naming this entry directly",
        },
    ]
}

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
    axum::extract::Query(query): axum::extract::Query<CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_PLUGIN).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let mut plugins = view_models::plugin_rows(
        catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let search = query.q.unwrap_or_default();
    plugins.retain(|p| matches(&[&p.id, &p.name, &p.description, &p.category], &search));
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    match sort_key.as_str() {
        "status" => apply_direction(&mut plugins, dir, |a, b| a.enabled.cmp(&b.enabled)),
        "members" => apply_direction(&mut plugins, dir, |a, b| a.skills_count.cmp(&b.skills_count)),
        "visibility" => apply_direction(&mut plugins, dir, |a, b| {
            a.visibility.is_public.cmp(&b.visibility.is_public)
        }),
        "grants" => apply_direction(&mut plugins, dir, |a, b| {
            a.assignment_count.cmp(&b.assignment_count)
        }),
        _ => apply_direction(&mut plugins, dir, |a, b| b.name.cmp(&a.name)),
    }

    let enabled = plugins.iter().filter(|p| p.enabled).count();
    let public = plugins.iter().filter(|p| p.visibility.is_public).count();
    let skills: usize = plugins.iter().map(|p| p.skills_count).sum();
    let servers: usize = plugins.iter().map(|p| p.mcp_count).sum();
    let ungranted = plugins.iter().filter(|p| p.assignment_count == 0).count();
    let page = view::PluginsPageData {
        page: "plugins",
        title: "Plugins",
        subtitle: "Collections. A plugin bundles skills, MCP servers, agents and hooks, and a marketplace ships plugins.",
        breadcrumbs: vec![BreadcrumbView::current("Plugins")],
        kpis: vec![
            kpi("Plugins", plugins.len().to_string(), format!("{enabled} enabled"), ""),
            kpi("Skills carried", skills.to_string(), "across all plugins".to_owned(), ""),
            kpi("MCP servers carried", servers.to_string(), "across all plugins".to_owned(), ""),
            kpi("Public", public.to_string(), "no rule restricts them".to_owned(), if public > 0 { "warn" } else { "ok" }),
            kpi("No direct grant", ungranted.to_string(), "reachable only via a marketplace".to_owned(), ""),
            kpi("Disabled", (plugins.len() - enabled).to_string(), "declared but switched off".to_owned(), ""),
        ],
        sort_headers: sort_headers(
            "/admin/plugins",
            &entry_columns("Members", "Skills, MCP servers and agents this plugin carries"),
            &sort_key,
            dir,
            &preserved_search(&search),
        ),
        plugins_count: plugins.len(),
        plugins,
        access_control_url: "/admin/access-control?entity_type=plugin",
        search,
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
    axum::extract::Query(query): axum::extract::Query<CatalogListQuery>,
) -> AdminHtmlResult<Response> {
    admin_only(&user_ctx)?;
    let path = shared::get_services_path()?;

    let catalog = data::load_catalog(&path, &user_ctx.roles);
    let counts = assignment_counts_by_type(&pool, ENTITY_SKILL).await;
    let (manifests, rules) = visibility_inputs(&pool, &path).await;
    let mut skills = view_models::skill_rows(
        &catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    let search = query.q.unwrap_or_default();
    skills.retain(|s| matches(&[&s.id, &s.name, &s.description], &search));
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    match sort_key.as_str() {
        "status" => apply_direction(&mut skills, dir, |a, b| a.enabled.cmp(&b.enabled)),
        "members" => apply_direction(&mut skills, dir, |a, b| a.plugin_count.cmp(&b.plugin_count)),
        "visibility" => apply_direction(&mut skills, dir, |a, b| {
            a.visibility.is_public.cmp(&b.visibility.is_public)
        }),
        "grants" => apply_direction(&mut skills, dir, |a, b| {
            a.assignment_count.cmp(&b.assignment_count)
        }),
        _ => apply_direction(&mut skills, dir, |a, b| b.name.cmp(&a.name)),
    }

    let enabled = skills.iter().filter(|s| s.enabled).count();
    let public = skills.iter().filter(|s| s.visibility.is_public).count();
    let orphaned = skills.iter().filter(|s| s.plugin_count == 0).count();
    let ungranted = skills.iter().filter(|s| s.assignment_count == 0).count();
    let page = view::SkillsPageData {
        page: "skills",
        title: "Skills",
        subtitle: "The instruction sets people invoke. A skill reaches someone through the plugins that include it.",
        breadcrumbs: vec![BreadcrumbView::current("Skills")],
        kpis: vec![
            kpi("Skills", skills.len().to_string(), format!("{enabled} enabled"), ""),
            kpi("In no plugin", orphaned.to_string(), "declared but nothing ships them".to_owned(), if orphaned > 0 { "warn" } else { "ok" }),
            kpi("Public", public.to_string(), "no rule restricts them".to_owned(), if public > 0 { "warn" } else { "ok" }),
            kpi("No direct grant", ungranted.to_string(), "reachable only via a plugin".to_owned(), ""),
            kpi("Disabled", (skills.len() - enabled).to_string(), "declared but switched off".to_owned(), ""),
            kpi("Plugins", manifests.len().to_string(), "marketplaces that can carry them".to_owned(), ""),
        ],
        sort_headers: sort_headers(
            "/admin/skills",
            &entry_columns("Plugins", "How many plugins include this skill"),
            &sort_key,
            dir,
            &preserved_search(&search),
        ),
        skills_count: skills.len(),
        skills,
        access_control_url: "/admin/access-control?entity_type=skill",
        search,
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

fn admin_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

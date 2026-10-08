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
//! plugin ↔ member relationship in both directions, and every detail page
//! carries the shared "Who gets this" panel — the one place an entity's
//! access is read and edited. The catalog itself is read-only: operators
//! edit `services/*.yaml` and restart.
//!
//! The `q=` filter is applied server-side rather than as a class toggle, so
//! the row count, the empty state and the URL always agree.

mod access;
mod data;
mod detail;
mod entries;
pub(crate) mod marketplaces;
pub(crate) mod mcp;
pub(crate) mod sorting;
pub(crate) use detail::{plugin_detail_page, skill_detail_page};
mod view;
mod view_models;
mod visibility;

use std::cmp::Ordering;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::sync::provenance::{
    OwnedKind, Provenance, owned_by,
};
use systemprompt_web_admin_core::repositories::sync::sources::build_sources;
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{
    ENTITY_PLUGIN, ENTITY_SKILL, MarketplaceContext, UserContext,
};

use entries::entry_columns;
use sorting::{apply_direction, direction, matches, preserved_search, sort_headers};
use systemprompt_web_admin_core::ssr::helpers::render_typed_page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use view::{PluginListRow, SkillListRow, assignment_counts_by_type};
use view_models::VisibilityInput;

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct CatalogListQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub q: Option<String>,
    pub tab: Option<String>,
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
    let (manifests, rules) = visibility::visibility_inputs(&pool, &path).await;
    let mut plugins = view_models::plugin_rows(
        catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    stamp_sources(&mut plugins, OwnedKind::Plugin);
    let search = query.q.unwrap_or_default();
    plugins.retain(|p| {
        matches(
            &[p.id.as_str(), &p.name, &p.description, &p.category],
            &search,
        )
    });
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    apply_direction(&mut plugins, dir, plugin_order(&sort_key));

    let page = view::PluginsPageData {
        page: "plugins",
        title: "Plugins",
        subtitle: "Collections. A plugin bundles skills, MCP servers, agents and hooks, and a marketplace ships plugins.",
        breadcrumbs: vec![BreadcrumbView::current("Plugins")],
        kpis: entries::plugin_kpis(&plugins),
        sort_headers: sort_headers(
            "/admin/plugins",
            &entry_columns(
                "Members",
                "Skills, MCP servers and agents this plugin carries",
            ),
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
    let (manifests, rules) = visibility::visibility_inputs(&pool, &path).await;
    let mut skills = view_models::skill_rows(
        &catalog,
        &counts,
        &VisibilityInput {
            manifests: &manifests,
            rules: &rules,
        },
    );
    stamp_sources(&mut skills, OwnedKind::Skill);
    let search = query.q.unwrap_or_default();
    skills.retain(|s| matches(&[&s.id, &s.name, &s.description], &search));
    let sort_key = query.sort.unwrap_or_else(|| "name".to_owned());
    let dir = direction(query.dir.as_deref());
    apply_direction(&mut skills, dir, skill_order(&sort_key));

    let page = view::SkillsPageData {
        page: "skills",
        title: "Skills",
        subtitle: "The instruction sets people invoke. A skill reaches someone through the plugins that include it.",
        breadcrumbs: vec![BreadcrumbView::current("Skills")],
        kpis: entries::skill_kpis(&skills, manifests.len()),
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

type RowOrder<T> = fn(&T, &T) -> Ordering;

fn plugin_order(sort_key: &str) -> RowOrder<PluginListRow> {
    match sort_key {
        "status" => |a, b| a.enabled.cmp(&b.enabled),
        "members" => |a, b| a.skills_count.cmp(&b.skills_count),
        "visibility" => |a, b| a.visibility.is_public.cmp(&b.visibility.is_public),
        "grants" => |a, b| a.assignment_count.cmp(&b.assignment_count),
        _ => |a, b| b.name.cmp(&a.name),
    }
}

fn skill_order(sort_key: &str) -> RowOrder<SkillListRow> {
    match sort_key {
        "status" => |a, b| a.enabled.cmp(&b.enabled),
        "members" => |a, b| a.plugin_count.cmp(&b.plugin_count),
        "visibility" => |a, b| a.visibility.is_public.cmp(&b.visibility.is_public),
        "grants" => |a, b| a.assignment_count.cmp(&b.assignment_count),
        _ => |a, b| b.name.cmp(&a.name),
    }
}

trait SourceStamped {
    fn source_key(&self) -> &str;
    fn stamp(&mut self, provenance: Provenance);
}

impl SourceStamped for PluginListRow {
    fn source_key(&self) -> &str {
        self.id.as_str()
    }

    fn stamp(&mut self, provenance: Provenance) {
        self.source = provenance.source;
        self.source_tone = provenance.tone;
    }
}

impl SourceStamped for SkillListRow {
    fn source_key(&self) -> &str {
        &self.id
    }

    fn stamp(&mut self, provenance: Provenance) {
        self.source = provenance.source;
        self.source_tone = provenance.tone;
    }
}

fn stamp_sources<T: SourceStamped>(rows: &mut [T], kind: OwnedKind) {
    let sources = match build_sources() {
        Ok(sources) => Some(sources),
        Err(error) => {
            tracing::warn!(%error, "catalog sources unreadable; rows stay base");
            None
        },
    };
    for row in rows {
        let provenance = sources
            .as_ref()
            .map_or_else(Provenance::base, |s| owned_by(s, kind, row.source_key()));
        row.stamp(provenance);
    }
}

fn admin_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
}

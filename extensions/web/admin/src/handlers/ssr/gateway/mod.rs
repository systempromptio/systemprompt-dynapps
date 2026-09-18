//! `/admin/gateway` — the model-routing table and the settings above it.
//!
//! This is the one editing surface in the platform section: everything else on
//! these pages is read from YAML and changed by an operator with a text editor.
//! The gateway is different because route order is policy — the first pattern
//! that matches wins — and reordering a file by hand while traffic is flowing
//! is how a rewrite ends up pointing at the wrong provider.
//!
//! Every mutation goes through the existing JSON API (`PATCH /gateway`,
//! `POST|PATCH|DELETE /gateway/routes`). Routes are read from the
//! `gateway_routes` table, which every write lands in before the file's
//! `routes:` sequence is regenerated from it; core boots from that file, so
//! a saved route is dispatched at the next restart and the page says so.
//! `pricing`/`when`/`requires` blocks the form does not render are carried
//! through untouched. The page never writes YAML itself.

mod models;
mod rows;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::pickable_users::list_pickable_users;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories;
use crate::repositories::config::gateway::{RouteLabels, get_route_labels_from_services};
use crate::templates::AdminTemplateEngine;
use crate::types::{GatewayConfigView, MarketplaceContext, UserContext};

use rows::{kpis, load_surfaces, resolved_only, route_rows};
use view::GatewayPageData;

const ENTITY_GATEWAY_ROUTE: &str = "gateway_route";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct GatewayQuery {
    tab: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GatewayTab {
    Models,
    Routes,
    Resolved,
}

impl GatewayTab {
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("routes") => Self::Routes,
            Some("resolved") => Self::Resolved,
            _ => Self::Models,
        }
    }
}

// Why: four views, each a URL. Models is what an operator comes for —
// which providers are live and what reaches them; Routes is the ordered
// table the dispatcher reads, where order is edited; Resolved is what the
// dispatcher matches that the file does not list; Policies lives on its own
// page and is linked so the gateway reads as one thing.
fn view_tabs(active: GatewayTab) -> Vec<TabLinkView> {
    let link = |slug, label, href: &str, tab: Option<GatewayTab>| TabLinkView {
        slug,
        label,
        href: href.to_owned(),
        is_active: tab == Some(active),
        count: None,
    };
    vec![
        link(
            "models",
            "Models",
            "/admin/gateway",
            Some(GatewayTab::Models),
        ),
        link(
            "routes",
            "Routes",
            "/admin/gateway?tab=routes",
            Some(GatewayTab::Routes),
        ),
        link(
            "resolved",
            "Resolved only",
            "/admin/gateway?tab=resolved",
            Some(GatewayTab::Resolved),
        ),
        link("policies", "Policies", "/admin/gateway/policies", None),
    ]
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(crate::error::AdminError::Forbidden("Admin access required.".to_owned()).into())
}

pub(crate) async fn gateway_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GatewayQuery>,
) -> AdminHtmlResult<Response> {
    console_only(&user_ctx)?;

    let (config, load_error) = match super::super::gateway_config_view(&pool).await {
        Ok(config) => (config, String::new()),
        Err(e) => (GatewayConfigView::default(), e.to_string()),
    };
    let (resolved, catalog_error) =
        match repositories::config::gateway::dispatchable_routes_from_services() {
            Ok(routes) => (routes, String::new()),
            Err(e) => (Vec::new(), e.to_string()),
        };

    let labels: RouteLabels = get_route_labels_from_services()
        .inspect_err(|e| tracing::warn!(error = %e, "gateway: route labels unavailable"))
        .unwrap_or_default();
    let surfaces = load_surfaces(&labels);
    let grants = repositories::users::access_control::count_assignments_by_entity_type(
        &pool,
        ENTITY_GATEWAY_ROUTE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "gateway: route rule listing failed"))
    .unwrap_or_default();
    let dispatchable: Vec<String> = resolved.iter().map(|r| r.id.clone()).collect();

    let tab = GatewayTab::parse(query.tab.as_deref());
    let routes = route_rows(&config, &surfaces, &grants, &dispatchable, &labels);
    let extra = resolved_only(&routes, &resolved, &labels);
    let provider_cards = if tab == GatewayTab::Models {
        models::provider_cards(&labels, &routes, &grants)
    } else {
        Vec::new()
    };

    let page = GatewayPageData {
        page: "gateway",
        title: "Gateway",
        subtitle: "Which providers this instance can reach, the models they serve, and which requested model goes where.",
        breadcrumbs: vec![BreadcrumbView::current("Gateway")],
        enabled: config.enabled,
        auth_scheme: config.auth_scheme.clone(),
        inference_path_prefix: config.inference_path_prefix.clone(),
        source_path: config.source_path.clone(),
        kpis: kpis(&config, &routes, extra.len(), &surfaces),
        routes_count: routes.len(),
        routes,
        resolved_only_count: extra.len(),
        resolved_only: extra,
        providers_count: surfaces.providers.len(),
        providers: surfaces.providers,
        probe_users: list_pickable_users(&pool, None).await,
        provider_cards,
        load_error,
        catalog_error,
        runtime_note: repositories::sync::gateway_routes::RUNTIME_NOTE,
        sync_url: "/admin/sync?entity=gateway_route",
        tabs: view_tabs(tab),
        show_models: tab == GatewayTab::Models,
        show_routes: tab == GatewayTab::Routes,
        show_resolved: tab == GatewayTab::Resolved,
    };
    Ok(render_typed_page(
        &engine, "gateway", &page, &user_ctx, &mkt_ctx,
    ))
}

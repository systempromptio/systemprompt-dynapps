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
//!
//! Each of the four tabs is a URL: Overview (dispatch order and provider
//! health), Providers (each upstream opened into its models and routes),
//! Routes (the ordered dispatch table plus the resolved-only set) and
//! Settings (switches and the per-account probe), plus Sync, the
//! `gateway_routes` plane's code-against-database card. Policies is linked
//! as a last tab but lives on its own page.

mod models;
mod route_detail;
mod rows;
mod tabs;
mod view;

use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_web_admin_core::ssr::page::Page;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::repositories;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::repositories::gateway_catalog::{RouteLabels, get_route_labels};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::pickable_users::list_pickable_users;
use systemprompt_web_admin_core::ssr::sync_plane::plane_card_by_id;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::types::{GatewayConfigView, GatewayRouteView, UserContext};

pub(crate) use route_detail::gateway_route_page;
use rows::{RouteRowInputs, Surfaces, kpis, load_surfaces, resolved_only, route_rows};
use tabs::{GatewayTab, view_tabs};
use view::{GatewayPageData, GatewaySyncView};

const ENTITY_GATEWAY_ROUTE: &str = "gateway_route";

const NAMES_NOTE: &str = "A route's name and description are written to services/ai/gateway.yaml with the route; a provider's display name lives in services/ai/providers.yaml. The rest of the console reads them from the booted services, so a name edited here shows elsewhere after the same restart.";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct GatewayQuery {
    tab: Option<String>,
}

fn console_only(user_ctx: &UserContext) -> AdminHtmlResult<()> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(systemprompt_web_admin_core::error::AdminError::Forbidden(
        "Admin access required.".to_owned(),
    )
    .into())
}

struct GatewayRead {
    config: GatewayConfigView,
    config_error: String,
    resolved: Vec<GatewayRouteView>,
    catalog_error: String,
    labels: RouteLabels,
    surfaces: Surfaces,
    grants_by_route: HashMap<String, i64>,
}

async fn load_gateway(
    pool: &PgPool,
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    faults: &mut Faults,
) -> GatewayRead {
    let (config, config_error) = match super::super::gateway_config_view(pool).await {
        Ok(config) => (config, String::new()),
        Err(e) => (GatewayConfigView::default(), e.to_string()),
    };
    let (resolved, catalog_error) =
        match systemprompt_web_admin_core::repositories::gateway_catalog::dispatchable_routes(
            &services.config(),
        ) {
            Ok(routes) => (routes, String::new()),
            Err(e) => (Vec::new(), e.to_string()),
        };
    let labels: RouteLabels = faults.take("route labels", get_route_labels(&services.config()));
    let surfaces = load_surfaces(&labels);
    let grants_by_route = faults.take(
        "route grants",
        systemprompt_web_admin_core::repositories::entitlement::count_assignments_by_entity_type(
            pool,
            ENTITY_GATEWAY_ROUTE,
        )
        .await,
    );
    GatewayRead {
        config,
        config_error,
        resolved,
        catalog_error,
        labels,
        surfaces,
        grants_by_route,
    }
}

async fn gateway_view(
    pool: &PgPool,
    read: GatewayRead,
    tab: GatewayTab,
    faults: &mut Faults,
) -> GatewayPageData {
    let GatewayRead {
        config,
        config_error,
        resolved,
        catalog_error,
        labels,
        surfaces,
        grants_by_route: grants,
    } = read;
    let dispatchable: Vec<String> = resolved.iter().map(|r| r.id.clone()).collect();
    let routes = route_rows(
        RouteRowInputs {
            config: &config,
            surfaces: &surfaces,
            grants: &grants,
            dispatchable: &dispatchable,
            labels: &labels,
        },
        faults,
    );
    let extra = resolved_only(&routes, &resolved, &labels);
    let provider_cards = if matches!(tab, GatewayTab::Overview | GatewayTab::Providers) {
        models::provider_cards(&labels, &routes, &grants)
    } else {
        Vec::new()
    };
    let dispatch = if tab == GatewayTab::Overview {
        models::dispatch_rows(&labels, &routes, &grants)
    } else {
        Vec::new()
    };

    let routes_count = routes.len();
    let providers_count = surfaces.providers.len();
    GatewayPageData {
        page: "gateway",
        title: "Gateway",
        subtitle: "Which providers this instance can reach, the models they serve, and which requested model goes where.",
        breadcrumbs: vec![BreadcrumbView::current("Gateway")],
        enabled: config.enabled,
        auth_scheme: config.auth_scheme.clone(),
        inference_path_prefix: config.inference_path_prefix.clone(),
        source_path: config.source_path.clone(),
        kpis: kpis(&config, &routes, extra.len(), &surfaces),
        routes_count,
        routes,
        resolved_only_count: extra.len(),
        resolved_only: extra,
        providers_count,
        providers: surfaces.providers,
        probe_users: faults.take("probe users", list_pickable_users(pool, None).await),
        provider_cards,
        dispatch_count: dispatch.len(),
        dispatch,
        load_error: config_error,
        catalog_error,
        runtime_note: repositories::sync_planes::gateway_routes::RUNTIME_NOTE,
        sync: None,
        names_note: NAMES_NOTE,
        tabs: view_tabs(tab, routes_count, providers_count),
        show_overview: tab == GatewayTab::Overview,
        show_providers: tab == GatewayTab::Providers,
        show_routes: tab == GatewayTab::Routes,
        show_settings: tab == GatewayTab::Settings,
    }
}

pub(crate) async fn gateway_page(
    shell: Page,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GatewayQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = shell;
    console_only(&user_ctx)?;
    let mut faults = Faults::new();
    let read = load_gateway(&pool, &services, &mut faults).await;
    let tab = GatewayTab::parse(query.tab.as_deref());
    let mut page = gateway_view(&pool, read, tab, &mut faults).await;
    if tab == GatewayTab::Sync {
        let plane = repositories::sync_planes::gateway_routes::PLANE_ID;
        page.sync = plane_card_by_id(&pool, plane, "")
            .await?
            .map(|card| GatewaySyncView {
                card,
                can_write: user_ctx.is_admin,
            });
    }
    Ok(render_page_with_faults(
        &engine,
        "gateway",
        &page,
        Viewer::new(&user_ctx, &mkt_ctx),
        &faults,
    ))
}

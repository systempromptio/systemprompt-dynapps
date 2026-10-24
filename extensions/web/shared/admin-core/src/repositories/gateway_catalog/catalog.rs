//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the rules.yaml projection it feeds, and the dashboard handlers
//! — derives the set here, from the same resolved catalog the gateway
//! dispatches by, so no two of them can disagree about which ids are real.
//! It includes the catalog route, the id access control evaluates for a
//! request no declared route matched, which `gateway.yaml` never lists. The
//! routes and the catalog they resolve against are services configuration
//! (`services/ai/*.yaml`), read from the `ServicesBootstrap` cell the process
//! booted with.

use systemprompt::governance::authz::{EntityKind, RegisteredEntities};
use systemprompt::identifiers::RouteId;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::manifest::ServicesConfig;
use systemprompt::manifest::services::{CatalogContext, ResolvedCatalog};
use systemprompt_web_shared::error::MarketplaceError;

use super::yaml_io::route_view;
use crate::types::GatewayRouteView;

pub const CATALOG_ROUTE_LABEL: &str = "Every catalog model";

pub fn resolved_catalog(services: &ServicesConfig) -> Result<ResolvedCatalog, MarketplaceError> {
    services
        .resolve_catalog(&CatalogContext::default())
        .map_err(|e| MarketplaceError::Internal(format!("the AI catalog does not resolve: {e}")))
}

pub fn dispatchable_route_ids(services: &ServicesConfig) -> Vec<String> {
    resolved_catalog(services)
        .map(|catalog| catalog.route_ids())
        .unwrap_or_default()
        .iter()
        .map(RouteId::as_str)
        .map(str::to_owned)
        .collect()
}

pub fn dispatchable_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    let catalog = resolved_catalog(services)?;
    let mut routes: Vec<GatewayRouteView> =
        catalog.routes().iter().cloned().map(route_view).collect();
    let catalog_id = catalog.catalog_route_id().as_str();
    if !routes.iter().any(|route| route.id == catalog_id) {
        routes.push(GatewayRouteView {
            id: catalog_id.to_owned(),
            name: Some(CATALOG_ROUTE_LABEL.to_owned()),
            description: Some(
                "Any request no declared route matches: each model is served from its own \
                 deployments."
                    .to_owned(),
            ),
            models: vec!["*".to_owned()],
            ..GatewayRouteView::default()
        });
    }
    Ok(routes)
}

pub fn dispatchable_routes_from_services() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    dispatchable_routes(ServicesBootstrap::get()?)
}

pub fn client_facing_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    let catalog = resolved_catalog(services)?;
    Ok(retain_client_facing(dispatchable_routes(services)?, &catalog))
}

/// Keeps the routes that reach at least one model some client lists.
#[must_use]
pub fn retain_client_facing(
    routes: Vec<GatewayRouteView>,
    catalog: &ResolvedCatalog,
) -> Vec<GatewayRouteView> {
    routes
        .into_iter()
        .filter(|route| {
            catalog
                .models()
                .filter(|model| model.is_advertised())
                .any(|model| route_reaches(route, model.id.as_str()))
        })
        .collect()
}

/// Whether a request for `model` can land on this route.
#[must_use]
pub fn route_reaches(route: &GatewayRouteView, model: &str) -> bool {
    route.alias.as_ref().is_some_and(|alias| alias.model == model)
        || route
            .models
            .iter()
            .any(|glob| super::matching::glob_match(glob, model))
}

pub fn client_facing_routes_from_services() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    client_facing_routes(ServicesBootstrap::get()?)
}

#[must_use]
pub fn registered_routes(route_ids: &[String]) -> RegisteredEntities {
    if route_ids.is_empty() {
        RegisteredEntities::default()
    } else {
        RegisteredEntities::new().with_kind(EntityKind::GatewayRoute, route_ids.iter().cloned())
    }
}

pub fn registered_routes_from_services() -> Result<RegisteredEntities, MarketplaceError> {
    let services = ServicesBootstrap::get()?;
    Ok(registered_routes(&dispatchable_route_ids(services)))
}

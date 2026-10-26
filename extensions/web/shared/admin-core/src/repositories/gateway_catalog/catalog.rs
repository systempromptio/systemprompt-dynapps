//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the rules.yaml projection it feeds, and the dashboard handlers
//! — derives the set here, from the same resolved catalog the gateway
//! dispatches by, so no two of them can disagree about which ids are real.
//! A request no declared route matches has no route id: core decides it on
//! the `model/<id>` entity it asked for, so no synthetic route stands in for
//! it here. The routes and the catalog they resolve against are services
//! configuration (`services/ai/*.yaml`), read from the services config the
//! process booted with.

use systemprompt::governance::authz::{EntityKind, RegisteredEntities};
use systemprompt::identifiers::RouteId;
use systemprompt::manifest::ServicesConfig;
use systemprompt::manifest::services::{CatalogContext, ResolvedCatalog};
use systemprompt_web_shared::error::MarketplaceError;

use super::yaml_io::route_view;
use crate::types::GatewayRouteView;

pub fn resolved_catalog(services: &ServicesConfig) -> Result<ResolvedCatalog, MarketplaceError> {
    Ok(services.resolve_catalog(&CatalogContext::default())?)
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
    Ok(resolved_catalog(services)?
        .routes()
        .iter()
        .cloned()
        .map(route_view)
        .collect())
}

pub fn client_facing_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    let catalog = resolved_catalog(services)?;
    Ok(retain_client_facing(
        dispatchable_routes(services)?,
        &catalog,
    ))
}

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

#[must_use]
pub fn route_reaches(route: &GatewayRouteView, model: &str) -> bool {
    route
        .alias
        .as_ref()
        .is_some_and(|alias| alias.model == model)
        || route
            .models
            .iter()
            .any(|glob| super::matching::glob_match(glob, model))
}

#[must_use]
pub fn registered_routes(route_ids: &[String]) -> RegisteredEntities {
    if route_ids.is_empty() {
        RegisteredEntities::default()
    } else {
        RegisteredEntities::new().with_kind(EntityKind::GatewayRoute, route_ids.iter().cloned())
    }
}

#[must_use]
pub fn registered_routes_from_services(services: &ServicesConfig) -> RegisteredEntities {
    registered_routes(&dispatchable_route_ids(services))
}

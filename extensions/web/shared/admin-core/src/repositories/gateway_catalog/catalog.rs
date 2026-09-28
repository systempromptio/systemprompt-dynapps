//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row â the governance
//! bootstrap, the rules.yaml projection it feeds, and the dashboard handlers
//! â derives the set here, from the same `dispatchable_route_ids` the gateway
//! dispatches by, so no two of them can disagree about which ids are real. It
//! includes the synthesized catch-all route, which the gateway YAML never
//! lists. The routes and the registry they resolve against are services
//! configuration (`services/ai/gateway.yaml`, `services/ai/providers.yaml`),
//! read from the `ServicesBootstrap` cell the process booted with.

use systemprompt::identifiers::RouteId;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::models::services::ProviderRegistry;
use systemprompt::security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

pub fn dispatchable_route_ids(services: &ServicesConfig) -> Vec<String> {
    services
        .gateway
        .as_ref()
        .map(|gateway| gateway.dispatchable_route_ids(&services.providers))
        .unwrap_or_default()
        .iter()
        .map(RouteId::as_str)
        .map(str::to_owned)
        .collect()
}

pub fn dispatchable_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    let config = services.gateway_config().ok_or_else(missing_gateway)?;
    Ok(config
        .candidate_routes(&services.providers)
        .map(|route| {
            let route = route.into_owned();
            GatewayRouteView {
                id: route.id.as_str().to_owned(),
                name: route.name,
                description: route.description,
                model_pattern: route.model_pattern,
                provider: route.provider.as_str().to_owned(),
                upstream_model: route.upstream_model,
                extra_headers: route.extra_headers.into_iter().collect(),
                pricing: None,
                when: None,
                requires: None,
                fallback_provider: route.fallback_provider.map(|p| p.as_str().to_owned()),
                fallback_upstream_model: route.fallback_upstream_model,
            }
        })
        .collect())
}

fn missing_gateway() -> MarketplaceError {
    MarketplaceError::Internal(
        "no gateway configuration in the services tree — expected a `gateway:` block in \
         services/ai/gateway.yaml, included from services/config/config.yaml"
            .to_owned(),
    )
}

pub fn dispatchable_routes_from_services() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    dispatchable_routes(ServicesBootstrap::get()?)
}

pub fn client_facing_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    Ok(retain_client_facing(
        dispatchable_routes(services)?,
        &services.providers,
    ))
}

#[must_use]
pub fn retain_client_facing(
    routes: Vec<GatewayRouteView>,
    providers: &ProviderRegistry,
) -> Vec<GatewayRouteView> {
    routes
        .into_iter()
        .filter(|route| {
            providers
                .find_provider(&route.provider)
                .is_none_or(|entry| entry.surface.is_advertised())
        })
        .collect()
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

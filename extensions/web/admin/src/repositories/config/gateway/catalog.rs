//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the roles.yaml ingestion it feeds, and the dashboard handlers —
//! derives the set here, from the same `dispatchable_route_ids` the gateway
//! dispatches by, so no two of them can disagree about which ids are real. It
//! includes the synthesized catch-all route, which the profile YAML never
//! lists.

use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::RouteId;
use systemprompt::models::Profile;
use systemprompt::security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

pub fn dispatchable_route_ids(profile: &Profile) -> Vec<String> {
    profile
        .gateway
        .as_ref()
        .map(|gateway| gateway.dispatchable_route_ids(&profile.providers))
        .unwrap_or_default()
        .iter()
        .map(RouteId::as_str)
        .map(str::to_owned)
        .collect()
}

// Why: the same routes as `dispatchable_route_ids`, carrying the metadata the
// access-control views label and filter by. Read views that decide *access*
// must use this rather than the profile YAML `get_gateway_config` reads: that
// YAML omits the synthesized catch-all, so a grant on it would be invisible —
// and therefore unreachable — from any surface built on the file.
pub fn dispatchable_routes(profile: &Profile) -> Vec<GatewayRouteView> {
    let Some(config) = profile.gateway.as_ref().and_then(|g| g.resolved()) else {
        return Vec::new();
    };
    config
        .candidate_routes(&profile.providers)
        .map(|route| {
            let route = route.into_owned();
            GatewayRouteView {
                id: route.id.as_str().to_owned(),
                model_pattern: route.model_pattern,
                provider: route.provider.as_str().to_owned(),
                upstream_model: route.upstream_model,
                extra_headers: route.extra_headers.into_iter().collect(),
                // Why: read-only projection for the access-control views —
                // nothing here is ever written back to the profile, so the
                // editor's passthrough fields are dropped rather than
                // round-tripped through a second representation.
                pricing: None,
                when: None,
                requires: None,
            }
        })
        .collect()
}

pub fn dispatchable_routes_from_profile() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    Ok(dispatchable_routes(ProfileBootstrap::get()?))
}

// Why: an empty set is a profile without a gateway, not a declaration that no
// route exists — enforcing it would reject every route grant in roles.yaml.
// Such a profile enforces nothing, and the boot job likewise leaves its
// catalog untouched.
#[must_use]
pub fn registered_routes(route_ids: &[String]) -> RegisteredEntities {
    if route_ids.is_empty() {
        RegisteredEntities::default()
    } else {
        RegisteredEntities::new().with_kind(EntityKind::GatewayRoute, route_ids.iter().cloned())
    }
}

pub fn registered_routes_from_profile() -> Result<RegisteredEntities, MarketplaceError> {
    let profile = ProfileBootstrap::get()?;
    Ok(registered_routes(&dispatchable_route_ids(profile)))
}

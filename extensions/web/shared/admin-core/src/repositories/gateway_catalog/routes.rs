//! Route CRUD against the services YAML's `gateway.routes` sequence, and the
//! whole-sequence replace the `gateway_routes` table uses to regenerate it.
//!
//! Every route has a declared, stable id: a new route without one takes the
//! slug of its label, alias or first glob. An edit adds no field the operator
//! did not write, and the when/requires/strategy/context-fallback blocks the
//! form never round-trips are kept from disk. A cleared name posts as `""`
//! and is stored as absent.

use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::Value;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

use super::matching::slugify_pattern;
use super::yaml_io::{
    read_gateway_file, route_from_yaml, route_to_yaml, routes_seq_mut, write_gateway_file,
};

pub fn validate_route(route: &GatewayRouteView) -> Result<(), MarketplaceError> {
    let globs = route.models.iter().any(|glob| !glob.trim().is_empty());
    if !globs && route.alias.is_none() {
        return Err(MarketplaceError::BadRequest(
            "a route needs models or an alias".into(),
        ));
    }
    if let Some(endpoint) = route
        .deployments
        .iter()
        .find(|d| d.endpoint.split_once('/').is_none())
    {
        return Err(MarketplaceError::BadRequest(format!(
            "deployment `{}` is not a provider/endpoint address",
            endpoint.endpoint
        )));
    }
    Ok(())
}

pub fn normalise_metadata(route: &mut GatewayRouteView) {
    let clean = |v: &mut Option<String>| {
        *v = v
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned);
    };
    clean(&mut route.name);
    clean(&mut route.description);
}

fn derived_id(route: &GatewayRouteView) -> String {
    let stem = route
        .name
        .as_deref()
        .or_else(|| route.alias.as_ref().map(|alias| alias.requested.as_str()))
        .or_else(|| route.models.first().map(String::as_str))
        .unwrap_or("route");
    slugify_pattern(stem)
}

pub fn create_route(
    gateway_path: &Path,
    route: &GatewayRouteView,
) -> Result<usize, MarketplaceError> {
    validate_route(route)?;
    let mut to_insert = route.clone();
    normalise_metadata(&mut to_insert);
    if to_insert.id.trim().is_empty() {
        to_insert.id = derived_id(&to_insert);
    }
    let mut doc = read_gateway_file(gateway_path)?;
    let new_index = {
        let routes = routes_seq_mut(&mut doc)?;
        for existing in routes.iter() {
            let effective_id_taken =
                route_from_yaml(existing).is_some_and(|r| r.id == to_insert.id);
            if effective_id_taken {
                return Err(MarketplaceError::BadRequest(format!(
                    "route id `{}` already exists",
                    to_insert.id
                )));
            }
        }
        routes.push(route_to_yaml(&to_insert)?);
        routes.len() - 1
    };
    write_gateway_file(gateway_path, &doc)?;
    Ok(new_index)
}

pub fn update_route(
    gateway_path: &Path,
    index: usize,
    route: &GatewayRouteView,
) -> Result<bool, MarketplaceError> {
    validate_route(route)?;
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        let mut merged = route.clone();
        normalise_metadata(&mut merged);
        if let Some(previous) = route_from_yaml(&routes[index]) {
            if merged.id.trim().is_empty() {
                merged.id = previous.id;
            }
            merged.when = merged.when.or(previous.when);
            merged.requires = merged.requires.or(previous.requires);
            merged.strategy = merged.strategy.or(previous.strategy);
            if merged.context_fallbacks.is_empty() {
                merged.context_fallbacks = previous.context_fallbacks;
            }
        }
        routes[index] = route_to_yaml(&merged)?;
    }
    write_gateway_file(gateway_path, &doc)?;
    Ok(true)
}

pub fn delete_route(gateway_path: &Path, index: usize) -> Result<bool, MarketplaceError> {
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        routes.remove(index);
    }
    write_gateway_file(gateway_path, &doc)?;
    Ok(true)
}

pub fn reorder_routes(gateway_path: &Path, order: &[usize]) -> Result<(), MarketplaceError> {
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        let n = routes.len();
        if order.len() != n {
            return Err(MarketplaceError::BadRequest(format!(
                "order has {} entries but there are {n} routes",
                order.len()
            )));
        }
        let mut seen = vec![false; n];
        for &i in order {
            if i >= n || seen[i] {
                return Err(MarketplaceError::BadRequest(
                    "order must be a permutation of route indices".into(),
                ));
            }
            seen[i] = true;
        }
        let mut by_index: BTreeMap<usize, Value> =
            std::mem::take(routes).into_iter().enumerate().collect();
        for &i in order {
            if let Some(v) = by_index.remove(&i) {
                routes.push(v);
            }
        }
    }
    write_gateway_file(gateway_path, &doc)?;
    Ok(())
}

pub fn replace_routes(
    gateway_path: &Path,
    routes: &[GatewayRouteView],
) -> Result<(), MarketplaceError> {
    for route in routes {
        validate_route(route)?;
    }
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let seq = routes_seq_mut(&mut doc)?;
        seq.clear();
        for route in routes {
            seq.push(route_to_yaml(route)?);
        }
    }
    write_gateway_file(gateway_path, &doc)
}

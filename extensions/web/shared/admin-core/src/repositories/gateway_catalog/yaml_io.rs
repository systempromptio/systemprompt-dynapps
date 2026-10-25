//! Gateway YAML read/write and route <-> YAML conversion.
//!
//! All mutation paths funnel through [`read_gateway_file`] /
//! [`write_gateway_file`] so the `gateway` block stays well-formed, and through
//! [`ensure_gateway_mut`] / [`routes_seq_mut`] which lazily create the block
//! and `routes` sequence.
//!
//! `serde_yaml` round-trips values, not text, so a naive write drops every
//! comment in the file. [`write_gateway_file`] carries the operator's header
//! comment across, and [`route_to_yaml`] writes only the fields a route sets,
//! so an admin edit changes the field it edited and nothing else.

use std::path::Path;

use serde_yaml::{Mapping, Value};
use systemprompt_web_shared::error::MarketplaceError;

use systemprompt::manifest::services::GatewayRoute;

use crate::types::{GatewayRouteView, RouteAliasView, RouteDeploymentView};

pub fn read_gateway_file(gateway_path: &Path) -> Result<Value, MarketplaceError> {
    let content = std::fs::read_to_string(gateway_path)?;
    let doc: Value = serde_yaml::from_str(&content)?;
    Ok(doc)
}

pub(super) fn write_gateway_file(gateway_path: &Path, doc: &Value) -> Result<(), MarketplaceError> {
    let header = leading_comment_header(gateway_path);
    let yaml_str = format!("{header}{}", serde_yaml::to_string(doc)?);
    std::fs::write(gateway_path, yaml_str)
        .map_err(|e| MarketplaceError::config_file(gateway_path.display().to_string(), e))?;
    Ok(())
}

fn leading_comment_header(gateway_path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(gateway_path) else {
        return String::new();
    };
    content
        .lines()
        .take_while(|line| line.trim_start().starts_with('#'))
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        })
}

pub fn route_from_yaml(val: &Value) -> Option<GatewayRouteView> {
    let route: GatewayRoute = serde_yaml::from_value(val.clone()).ok()?;
    Some(route_view(route))
}

#[must_use]
pub fn route_view(route: GatewayRoute) -> GatewayRouteView {
    GatewayRouteView {
        id: route.id.as_str().to_owned(),
        name: route.label,
        description: route.description,
        models: route.models.0,
        deployments: route
            .deployments
            .iter()
            .map(|deployment| RouteDeploymentView {
                endpoint: deployment.endpoint().to_string(),
                weight: deployment.weight(),
            })
            .collect(),
        alias: route.alias.map(|alias| RouteAliasView {
            requested: alias.requested.as_str().to_owned(),
            model: alias.model.as_str().to_owned(),
        }),
        when: route.when.and_then(|when| serde_yaml::to_value(when).ok()),
        requires: route
            .requires
            .and_then(|requires| serde_yaml::to_value(requires).ok()),
        strategy: (!route.strategy.is_ordered()).then(|| route.strategy.to_string()),
        context_fallbacks: route
            .context_fallbacks
            .iter()
            .map(|model| model.as_str().to_owned())
            .collect(),
    }
}

pub fn route_to_yaml(route: &GatewayRouteView) -> Result<Value, MarketplaceError> {
    let mut map = Mapping::new();
    map.insert(Value::from("id"), Value::from(route.id.trim().to_owned()));
    for (key, value) in [("label", &route.name), ("description", &route.description)] {
        if let Some(value) = value.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            map.insert(Value::from(key), Value::from(value.to_owned()));
        }
    }
    if !route.models.is_empty() {
        map.insert(Value::from("models"), serde_yaml::to_value(&route.models)?);
    }
    if let Some(when) = &route.when {
        map.insert(Value::from("when"), when.clone());
    }
    if let Some(requires) = &route.requires {
        map.insert(Value::from("requires"), requires.clone());
    }
    if !route.deployments.is_empty() {
        let entries = route.deployments.iter().map(deployment_to_yaml).collect();
        map.insert(Value::from("deployments"), Value::Sequence(entries));
    }
    if let Some(strategy) = route.strategy.as_deref().filter(|s| !s.trim().is_empty()) {
        map.insert(Value::from("strategy"), Value::from(strategy.to_owned()));
    }
    if !route.context_fallbacks.is_empty() {
        map.insert(
            Value::from("context_fallbacks"),
            serde_yaml::to_value(&route.context_fallbacks)?,
        );
    }
    if let Some(alias) = &route.alias {
        let mut entry = Mapping::new();
        entry.insert(
            Value::from("requested"),
            Value::from(alias.requested.clone()),
        );
        entry.insert(Value::from("model"), Value::from(alias.model.clone()));
        map.insert(Value::from("alias"), Value::Mapping(entry));
    }
    let value = Value::Mapping(map);
    serde_yaml::from_value::<GatewayRoute>(value.clone()).map_err(|source| {
        MarketplaceError::InvalidRoute {
            route: route.id.clone(),
            source,
        }
    })?;
    Ok(value)
}

fn deployment_to_yaml(deployment: &RouteDeploymentView) -> Value {
    deployment.weight.map_or_else(
        || Value::from(deployment.endpoint.clone()),
        |weight| {
            let mut entry = Mapping::new();
            entry.insert(
                Value::from("endpoint"),
                Value::from(deployment.endpoint.clone()),
            );
            entry.insert(Value::from("weight"), Value::from(weight));
            Value::Mapping(entry)
        },
    )
}

pub(super) fn ensure_gateway_mut(doc: &mut Value) -> Result<&mut Mapping, MarketplaceError> {
    let root = doc
        .as_mapping_mut()
        .ok_or_else(|| MarketplaceError::Internal("gateway YAML root is not a mapping".into()))?;
    if !root.contains_key(Value::from("gateway")) {
        root.insert(Value::from("gateway"), Value::Mapping(Mapping::new()));
    }
    root.get_mut(Value::from("gateway"))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| MarketplaceError::Internal("gateway block is not a mapping".into()))
}

pub(super) fn routes_seq_mut(doc: &mut Value) -> Result<&mut Vec<Value>, MarketplaceError> {
    let gw = ensure_gateway_mut(doc)?;
    if !gw.contains_key(Value::from("routes")) {
        gw.insert(Value::from("routes"), Value::Sequence(Vec::new()));
    }
    gw.get_mut(Value::from("routes"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| MarketplaceError::Internal("gateway.routes is not a sequence".into()))
}

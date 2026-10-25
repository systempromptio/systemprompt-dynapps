//! Gateway route and settings value types.
//!
//! A route is policy only: it aliases a requested id onto a catalog model,
//! pins or narrows the deployments a model is served from, or carries an
//! audit label and an access-control address. Routing itself is the
//! catalog's `deployments.yaml`. The view mirrors core's `GatewayRoute`
//! field for field, so a console round-trip never drops a block it does not
//! edit.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayRouteView {
    #[serde(default)]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deployments: Vec<RouteDeploymentView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<RouteAliasView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_fallbacks: Vec<String>,
}

impl GatewayRouteView {
    #[must_use]
    pub fn models_line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(alias) = &self.alias {
            parts.push(format!("{} → {}", alias.requested, alias.model));
        }
        if !self.models.is_empty() {
            parts.push(self.models.join(", "));
        }
        parts.join("; ")
    }
}

/// One `provider/endpoint` a route pins, with its weight under
/// `strategy: weighted`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RouteDeploymentView {
    pub endpoint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,
}

impl RouteDeploymentView {
    #[must_use]
    pub fn provider(&self) -> &str {
        self.endpoint
            .split_once('/')
            .map_or(self.endpoint.as_str(), |(provider, _)| provider)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RouteAliasView {
    pub requested: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GatewayConfigView {
    pub enabled: bool,
    pub auth_scheme: String,
    pub inference_path_prefix: String,
    pub routes: Vec<GatewayRouteView>,
    pub source_path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatewaySettingsRequest {
    pub enabled: Option<bool>,
    pub auth_scheme: Option<String>,
    pub inference_path_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRoutesRequest {
    pub order: Vec<usize>,
}

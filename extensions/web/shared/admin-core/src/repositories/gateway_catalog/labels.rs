//! Human-facing labels for gateway routes and providers.
//!
//! A route id is an address and a provider id is a key, so neither reads well
//! on a page. Every console surface that names a route —
//! the access-control rules, the audience grid, a person's matrix, the role
//! entitlements and the gateway page — resolves it here, so the same route
//! carries the same label everywhere. A declared `name` wins; otherwise the
//! label is derived from the route's models and the display name of the
//! provider its first deployment reaches, and the id is demoted to a subtitle
//! the page may still show.

use std::collections::HashMap;

use systemprompt::manifest::ServicesConfig;
use systemprompt::manifest::services::ResolvedCatalog;
use systemprompt::manifest::services::catalog::{ResolvedHop, ResolvedProvider};
use systemprompt_web_shared::error::MarketplaceError;

use super::catalog::{dispatchable_routes, resolved_catalog, route_reaches};
use crate::types::GatewayRouteView;

#[derive(Debug, Clone)]
pub struct RouteLabel {
    pub id: String,
    pub label: String,
    pub declared_name: bool,
    pub description: Option<String>,
    pub model_pattern: String,
    pub provider: String,
    pub provider_label: String,
    pub upstream: Option<String>,
    pub endpoint: String,
    pub surface: String,
    pub wire: String,
    pub fallback: Option<String>,
}

impl RouteLabel {
    #[must_use]
    pub fn subtitle(&self) -> String {
        let mut s = format!("{} → {}", self.model_pattern, self.provider_label);
        if let Some(upstream) = &self.upstream {
            s.push_str(&format!(" as {upstream}"));
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct ProviderLabel {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub endpoint: String,
    pub surface: String,
    pub wire: String,
    pub model_count: usize,
    pub advertised: bool,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RouteLabels {
    by_id: HashMap<String, RouteLabel>,
    pub routes: Vec<RouteLabel>,
    pub providers: Vec<ProviderLabel>,
}

impl RouteLabels {
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&RouteLabel> {
        self.by_id.get(id)
    }

    #[must_use]
    pub fn label_of(&self, id: &str) -> String {
        self.find(id)
            .map_or_else(|| id.to_owned(), |l| l.label.clone())
    }

    #[must_use]
    pub fn find_provider(&self, id: &str) -> Option<&ProviderLabel> {
        self.providers.iter().find(|p| p.id == id)
    }
}

pub fn get_route_labels(services: &ServicesConfig) -> Result<RouteLabels, MarketplaceError> {
    let catalog = resolved_catalog(services)?;
    let providers: Vec<ProviderLabel> = catalog
        .providers()
        .map(|provider| provider_label(&catalog, provider))
        .collect();
    let routes: Vec<RouteLabel> = dispatchable_routes(services)?
        .into_iter()
        .map(|route| route_label(&catalog, route))
        .collect();
    let by_id = routes.iter().map(|r| (r.id.clone(), r.clone())).collect();
    Ok(RouteLabels {
        by_id,
        routes,
        providers,
    })
}

fn route_hops<'a>(catalog: &'a ResolvedCatalog, route: &GatewayRouteView) -> Vec<&'a ResolvedHop> {
    let Some(model) = catalog
        .models()
        .find(|model| route_reaches(route, model.id.as_str()))
    else {
        return Vec::new();
    };
    let chain = catalog.deployments(model.id.as_str());
    if route.deployments.is_empty() {
        return chain.iter().collect();
    }
    route
        .deployments
        .iter()
        .filter_map(|pinned| {
            chain
                .iter()
                .find(|hop| hop.endpoint_ref().to_string() == pinned.endpoint)
        })
        .collect()
}

fn hop_label(catalog: &ResolvedCatalog, hop: &ResolvedHop) -> String {
    let label = derive_provider_label(
        hop.provider.as_str(),
        catalog.provider(hop.provider.as_str()),
    );
    if hop.upstream_model == hop.model.as_str() {
        label
    } else {
        format!("{label} as {}", hop.upstream_model)
    }
}

fn route_label(catalog: &ResolvedCatalog, route: GatewayRouteView) -> RouteLabel {
    let hops = route_hops(catalog, &route);
    let primary = hops.first().copied();
    let provider = primary
        .map(|hop| hop.provider.as_str().to_owned())
        .unwrap_or_default();
    let provider_label = derive_provider_label(&provider, catalog.provider(&provider));
    let declared = route
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());
    let fallback = (hops.len() > 1).then(|| {
        hops.iter()
            .skip(1)
            .map(|hop| hop_label(catalog, hop))
            .collect::<Vec<_>>()
            .join(", then ")
    });
    let mut clients: Vec<String> = catalog
        .models()
        .filter(|model| route_reaches(&route, model.id.as_str()))
        .flat_map(|model| model.clients.iter().map(ToString::to_string))
        .collect();
    clients.sort();
    clients.dedup();
    let model_pattern = route.models_line();
    RouteLabel {
        label: declared.map_or_else(
            || derive_route_label(&model_pattern, &provider_label),
            str::to_owned,
        ),
        declared_name: declared.is_some(),
        id: route.id,
        description: route.description,
        upstream: primary
            .filter(|hop| hop.upstream_model != hop.model.as_str())
            .map(|hop| hop.upstream_model.clone()),
        endpoint: primary.map(|hop| hop.url.clone()).unwrap_or_default(),
        surface: clients.join(", "),
        wire: primary.map(|hop| hop.wire.to_string()).unwrap_or_default(),
        model_pattern,
        provider,
        provider_label,
        fallback,
    }
}

fn provider_label(catalog: &ResolvedCatalog, provider: &ResolvedProvider) -> ProviderLabel {
    let id = provider.id.as_str();
    let endpoints: Vec<_> = catalog
        .endpoints()
        .filter(|endpoint| endpoint.provider.as_str() == id)
        .collect();
    let served: Vec<_> = catalog
        .models()
        .filter(|model| {
            catalog
                .deployments(model.id.as_str())
                .iter()
                .any(|hop| hop.provider.as_str() == id)
        })
        .collect();
    let mut clients: Vec<String> = served
        .iter()
        .flat_map(|model| model.clients.iter().map(ToString::to_string))
        .collect();
    clients.sort();
    clients.dedup();
    let join = |values: Vec<String>| values.join(", ");
    ProviderLabel {
        id: id.to_owned(),
        label: derive_provider_label(id, Some(provider)),
        description: provider.description.clone(),
        endpoint: join(endpoints.iter().map(|e| e.url.clone()).collect()),
        surface: clients.join(", "),
        wire: join(endpoints.iter().map(|e| e.wire.to_string()).collect()),
        model_count: served.len(),
        advertised: served.iter().any(|model| model.is_advertised()),
        models: served
            .iter()
            .map(|model| model.id.as_str().to_owned())
            .collect(),
    }
}

#[must_use]
pub fn derive_provider_label(id: &str, entry: Option<&ResolvedProvider>) -> String {
    if let Some(name) = entry
        .and_then(|e| e.display_name.as_deref())
        .map(str::trim)
        .filter(|n| !n.is_empty())
    {
        return name.to_owned();
    }
    match id {
        "anthropic" => "Anthropic".to_owned(),
        "openai" => "OpenAI".to_owned(),
        "openai-responses" => "OpenAI (Responses)".to_owned(),
        "gemini" => "Google Gemini".to_owned(),
        "vertex" => "Vertex AI".to_owned(),
        "vertex-maas" => "Vertex AI MaaS".to_owned(),
        "cerebras" => "Cerebras".to_owned(),
        other => title_case(other),
    }
}

#[must_use]
pub fn derive_route_label(model_pattern: &str, provider_label: &str) -> String {
    let pattern = model_pattern.trim();
    if pattern == "*" {
        return format!("Everything else · {provider_label}");
    }
    let stem = pattern.trim_matches('*');
    let family = title_case(stem.trim_matches(|c| c == '-' || c == '.'));
    if pattern.contains('*') {
        format!("{family} models · {provider_label}")
    } else {
        format!("{family} · {provider_label}")
    }
}

fn title_case(id: &str) -> String {
    id.split(['-', '_', '.'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

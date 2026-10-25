//! Shapes the gateway route table: provider surfaces, declared and
//! resolved-only rows, and the KPI strip above them.
//!
//! A route's own saved name wins over the label resolver, which reads the
//! booted services and so lags a console save until the next restart.

use super::models::matrix_url;
use super::view::{GatewayKpiView, GatewayRouteRow, ProviderOptionView, ResolvedOnlyRow};
use systemprompt_web_admin_core::repositories::gateway_catalog::{RouteLabels, derive_route_label};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::types::{GatewayConfigView, GatewayRouteView};

fn yaml_summary(value: Option<&serde_yaml::Value>, faults: &mut Faults) -> String {
    let Some(value) = value else {
        return String::new();
    };
    faults
        .take("route yaml", serde_yaml::to_string(value))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "---")
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) struct Surfaces {
    pub(super) providers: Vec<ProviderOptionView>,
}

impl Surfaces {
    pub(super) fn surface_of(&self, provider: &str) -> (String, &'static str) {
        match self.providers.iter().find(|p| p.name == provider) {
            None => ("unknown".to_owned(), "err"),
            Some(p) if p.advertised => (p.surface.clone(), "ok"),
            Some(p) => (p.surface.clone(), "warn"),
        }
    }
}

pub(super) fn load_surfaces(labels: &RouteLabels) -> Surfaces {
    let providers = labels
        .providers
        .iter()
        .map(|p| ProviderOptionView {
            name: p.id.clone(),
            label: p.label.clone(),
            surface: p.surface.clone(),
            model_count: p.model_count,
            advertised: p.advertised,
        })
        .collect();
    Surfaces { providers }
}

#[derive(Clone, Copy)]
pub(super) struct RouteRowInputs<'a> {
    pub(super) config: &'a GatewayConfigView,
    pub(super) surfaces: &'a Surfaces,
    pub(super) grants: &'a std::collections::HashMap<String, i64>,
    pub(super) dispatchable: &'a [String],
    pub(super) labels: &'a RouteLabels,
}

pub(super) fn route_rows(inputs: RouteRowInputs<'_>, faults: &mut Faults) -> Vec<GatewayRouteRow> {
    let RouteRowInputs {
        config,
        surfaces,
        grants,
        dispatchable,
        labels,
    } = inputs;
    let last = config.routes.len().saturating_sub(1);
    config
        .routes
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let resolved = labels.find(&route.id);
            let provider = resolved.map_or_else(
                || {
                    route
                        .deployments
                        .first()
                        .map(|d| d.provider().to_owned())
                        .unwrap_or_default()
                },
                |l| l.provider.clone(),
            );
            let (surface, surface_tone) = surfaces.surface_of(&provider);
            let provider_label = labels
                .find_provider(&provider)
                .map_or_else(|| provider.clone(), |p| p.label.clone());
            let model_pattern = route.models_line();
            let label = route
                .name
                .as_deref()
                .map(str::trim)
                .filter(|n| !n.is_empty())
                .map_or_else(
                    || derive_route_label(&model_pattern, &provider_label),
                    str::to_owned,
                );
            GatewayRouteRow {
                index,
                surface,
                surface_tone,
                name: route.name.clone().unwrap_or_default(),
                description: route.description.clone().unwrap_or_default(),
                label,
                provider_label,
                upstream_model: resolved
                    .and_then(|l| l.upstream.clone())
                    .unwrap_or_default(),
                requires: yaml_summary(route.requires.as_ref(), faults),
                when: yaml_summary(route.when.as_ref(), faults),
                has_pricing: false,
                header_count: 0,
                grants: grants.get(&route.id).copied().unwrap_or(0),
                dispatchable: dispatchable.contains(&route.id),
                is_first: index == 0,
                is_last: index == last,
                matrix_url: matrix_url(&route.id),
                id: route.id.clone(),
                model_pattern,
                provider,
            }
        })
        .collect()
}

pub(super) fn resolved_only(
    declared: &[GatewayRouteRow],
    resolved: &[GatewayRouteView],
    labels: &RouteLabels,
) -> Vec<ResolvedOnlyRow> {
    resolved
        .iter()
        .filter(|r| !declared.iter().any(|d| d.id == r.id))
        .map(|r| {
            let resolved = labels.find(&r.id);
            ResolvedOnlyRow {
                matrix_url: matrix_url(&r.id),
                label: labels.label_of(&r.id),
                id: r.id.clone(),
                model_pattern: r.models_line(),
                provider: resolved.map(|l| l.provider.clone()).unwrap_or_default(),
                upstream_model: resolved
                    .and_then(|l| l.upstream.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

pub(super) fn kpis(
    config: &GatewayConfigView,
    rows: &[GatewayRouteRow],
    resolved_extra: usize,
    surfaces: &Surfaces,
) -> Vec<GatewayKpiView> {
    let backend = rows.iter().filter(|r| r.surface.is_empty()).count();
    let unknown = rows.iter().filter(|r| r.surface == "unknown").count();
    let grants: i64 = rows.iter().map(|r| r.grants).sum();
    let governed = rows.iter().filter(|r| !r.requires.is_empty()).count();
    vec![
        GatewayKpiView {
            label: "Gateway",
            value: if config.enabled { "On" } else { "Off" }.to_owned(),
            note: format!("{} on {}", config.auth_scheme, config.inference_path_prefix),
            tone: if config.enabled { "ok" } else { "warn" },
        },
        GatewayKpiView {
            label: "Routes declared",
            value: rows.len().to_string(),
            note: format!("{resolved_extra} resolved but not in the file"),
            tone: "",
        },
        GatewayKpiView {
            label: "Providers",
            value: surfaces.providers.len().to_string(),
            note: format!("{backend} routes reaching no client-listed model"),
            tone: "",
        },
        GatewayKpiView {
            label: "Unknown provider",
            value: unknown.to_string(),
            note: "routes naming a provider the registry lacks".to_owned(),
            tone: if unknown > 0 { "err" } else { "ok" },
        },
        GatewayKpiView {
            label: "Governed routes",
            value: governed.to_string(),
            note: "carry a requires: classification".to_owned(),
            tone: "",
        },
        GatewayKpiView {
            label: "Route grants",
            value: grants.to_string(),
            note: "access-control rules on routes".to_owned(),
            tone: "",
        },
    ]
}

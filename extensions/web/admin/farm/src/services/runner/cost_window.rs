//! The gateway cost window a run's credential carries.
//!
//! Core's gateway reserves a worst-case estimate at admission (the body's
//! input plus the request's `max_tokens`, clamped to the model's output
//! ceiling, priced at the model's rate card) and refuses the request when
//! that reservation would pass the key's cost ceiling. With the ceiling set
//! to the agent's budget, an agent whose budget is below one worst-case
//! request (128k output tokens of Opus is $2.56) is refused on its first
//! turn. The window is therefore `max(budget, worst case) × 1.5`: wide enough
//! to admit one in-flight request, still a hard backstop. The agent's own
//! budget stays the true ceiling, enforced by the host's `--max-budget`.
//! The worst case prices the run's model when the catalog serves it, else
//! the dearest served model, else (no catalog loaded) one Opus-class request.
//! A model is priced at its dearest enabled deployment hop, since a hop's
//! pricing is what the gateway charges on that route.

use systemprompt::manifest::services::catalog::{ResolvedCatalog, ResolvedModel};

pub const COST_WINDOW_HEADROOM: f64 = 1.5;

pub const FALLBACK_WORST_CASE_USD: f64 = 2.56;

fn model_worst_case_usd(catalog: &ResolvedCatalog, model: &ResolvedModel) -> f64 {
    let output_per_million = catalog
        .enabled_hops(model.id.as_str())
        .map(|hop| hop.pricing.output_per_million)
        .reduce(f64::max)
        .unwrap_or(model.pricing.output_per_million);
    f64::from(model.limits.max_output_tokens) * output_per_million / 1_000_000.0
}

#[must_use]
pub fn worst_case_request_usd(catalog: &ResolvedCatalog, model: Option<&str>) -> Option<f64> {
    let found = model.and_then(|id| {
        catalog
            .model(id)
            .or_else(|| catalog.models().find(|m| m.upstream_model == id))
    });
    if let Some(found) = found {
        return Some(model_worst_case_usd(catalog, found));
    }
    catalog
        .models()
        .filter(|m| catalog.enabled_hops(m.id.as_str()).next().is_some())
        .map(|m| model_worst_case_usd(catalog, m))
        .reduce(f64::max)
}

#[must_use]
pub fn cost_window_usd(budget_usd: f64, worst_case_usd: f64) -> f64 {
    budget_usd.max(worst_case_usd) * COST_WINDOW_HEADROOM
}

#[must_use]
pub fn run_cost_window_usd(
    catalog: &ResolvedCatalog,
    budget_usd: Option<f64>,
    model: Option<&str>,
) -> Option<f64> {
    let budget = budget_usd?;
    let worst = worst_case_request_usd(catalog, model).unwrap_or(FALLBACK_WORST_CASE_USD);
    Some(cost_window_usd(budget, worst))
}

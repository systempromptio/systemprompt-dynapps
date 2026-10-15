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
//! The worst case prices the run's model when the registry serves it, else
//! the dearest served model, else (no registry loaded) one Opus-class request.

use systemprompt::manifest::services::ProviderRegistry;

pub const COST_WINDOW_HEADROOM: f64 = 1.5;

pub const FALLBACK_WORST_CASE_USD: f64 = 2.56;

#[must_use]
pub fn worst_case_request_usd(registry: &ProviderRegistry, model: Option<&str>) -> Option<f64> {
    let cost = |m: &systemprompt::manifest::services::ProviderModel| {
        f64::from(m.limits.max_output_tokens) * m.pricing.output_per_million / 1_000_000.0
    };
    let served = || registry.providers.iter().flat_map(|p| p.models.iter());
    if let Some(found) = model.and_then(|id| served().find(|m| m.matches(id))) {
        return Some(cost(found));
    }
    served().map(cost).reduce(f64::max)
}

#[must_use]
pub fn cost_window_usd(budget_usd: f64, worst_case_usd: f64) -> f64 {
    budget_usd.max(worst_case_usd) * COST_WINDOW_HEADROOM
}

#[must_use]
pub fn run_cost_window_usd(budget_usd: Option<f64>, model: Option<&str>) -> Option<f64> {
    let budget = budget_usd?;
    let worst = systemprompt::loader::ServicesBootstrap::get()
        .ok()
        .and_then(|services| worst_case_request_usd(&services.providers, model))
        .unwrap_or(FALLBACK_WORST_CASE_USD);
    Some(cost_window_usd(budget, worst))
}

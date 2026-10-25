//! The run credential's gateway cost window admits one worst-case request
//! even when the agent's budget is smaller than it.

use systemprompt::manifest::services::{CatalogContext, ResolvedCatalog};
use systemprompt_web_admin_farm::services::runner::cost_window::{
    COST_WINDOW_HEADROOM, cost_window_usd, worst_case_request_usd,
};

use crate::support::committed_services;

// Opus 5.5 is the dearest worst case in the committed catalog: 128K output
// tokens at $20/M; Haiku 5.5 is 128K at $0.50/M.
fn registry() -> ResolvedCatalog {
    committed_services()
        .resolve_catalog(&CatalogContext::default())
        .expect("the AI catalog resolves")
}

#[test]
fn a_one_dollar_budget_admits_a_worst_case_first_turn() {
    let worst = worst_case_request_usd(&registry(), Some("claude-opus-5-5")).expect("served");
    assert!((worst - 2.56).abs() < 1e-9);
    let window = cost_window_usd(1.0, worst);
    assert!(
        window >= worst,
        "the first turn's reservation fits the window"
    );
    assert!(2.56f64.mul_add(-COST_WINDOW_HEADROOM, window).abs() < 1e-9);
}

#[test]
fn a_large_budget_keeps_budget_times_headroom() {
    assert!((cost_window_usd(10.0, 2.56) - 15.0).abs() < 1e-9);
}

#[test]
fn the_run_model_prices_the_worst_case() {
    let worst = worst_case_request_usd(&registry(), Some("claude-haiku-5-5")).expect("served");
    assert!((worst - 0.064).abs() < 1e-9);
}

#[test]
fn an_unknown_model_uses_the_dearest_served() {
    let worst = worst_case_request_usd(&registry(), Some("nope")).expect("any");
    assert!((worst - 2.56).abs() < 1e-9);
    let none = worst_case_request_usd(&registry(), None).expect("any");
    assert!((none - 2.56).abs() < 1e-9);
}

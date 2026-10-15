//! The run credential's gateway cost window admits one worst-case request
//! even when the agent's budget is smaller than it.

use systemprompt::manifest::services::ProviderRegistry;
use systemprompt_web_admin_farm::services::runner::cost_window::{
    COST_WINDOW_HEADROOM, cost_window_usd, worst_case_request_usd,
};

const REGISTRY: &str = "- name: anthropic
  display_name: Anthropic
  wire: anthropic
  surface: anthropic
  endpoint: https://api.anthropic.com/v1
  api_key_secret: anthropic
  models:
  - id: opus
    pricing: { input_per_million: 4.0, output_per_million: 20.0 }
    limits: { context_window: 1000000, max_output_tokens: 128000 }
  - id: haiku
    pricing: { input_per_million: 0.1, output_per_million: 0.5 }
    limits: { context_window: 1000000, max_output_tokens: 128000 }
";

fn registry() -> ProviderRegistry {
    serde_yaml::from_str(REGISTRY).expect("registry parses")
}

#[test]
fn a_one_dollar_budget_admits_a_worst_case_first_turn() {
    let worst = worst_case_request_usd(&registry(), Some("opus")).expect("served");
    assert!((worst - 2.56).abs() < 1e-9);
    let window = cost_window_usd(1.0, worst);
    assert!(
        window >= worst,
        "the first turn's reservation fits the window"
    );
    assert!((window - 2.56 * COST_WINDOW_HEADROOM).abs() < 1e-9);
}

#[test]
fn a_large_budget_keeps_budget_times_headroom() {
    assert!((cost_window_usd(10.0, 2.56) - 15.0).abs() < 1e-9);
}

#[test]
fn the_run_model_prices_the_worst_case() {
    let worst = worst_case_request_usd(&registry(), Some("haiku")).expect("served");
    assert!((worst - 0.064).abs() < 1e-9);
}

#[test]
fn an_unknown_model_uses_the_dearest_served() {
    let worst = worst_case_request_usd(&registry(), Some("nope")).expect("any");
    assert!((worst - 2.56).abs() < 1e-9);
    let none = worst_case_request_usd(&registry(), None).expect("any");
    assert!((none - 2.56).abs() < 1e-9);
}

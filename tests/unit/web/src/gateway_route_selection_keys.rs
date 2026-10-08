//! The console's route view carries core's selection and context-window
//! keys — `strategy`, `weight` (route, fallbacks, scope chains) and
//! `context_fallbacks` — so a console save does not delete them.

use serde_yaml::Value;
use systemprompt_web_admin_core::repositories::gateway_catalog::{route_from_yaml, route_to_yaml};

const ROUTE: &str = r"model_pattern: claude-*
provider: vertex-eu
strategy: weighted
weight: 3
fallbacks:
- provider: vertex-us
  upstream_model: claude-sonnet-5
  weight: 2
- provider: anthropic
by_scope:
  dimension: project
  chains:
    acme:
      provider: vertex-acme
      strategy: least_busy
      weight: 4
      fallbacks:
      - provider: vertex-eu
        weight: 1
      context_fallbacks:
      - provider: vertex-long
context_fallbacks:
- provider: vertex-long
  upstream_model: claude-sonnet-5-1m
";

#[test]
fn selection_and_context_keys_survive_yaml_view_yaml() {
    let declared: Value = serde_yaml::from_str(ROUTE).expect("fixture");
    let view = route_from_yaml(&declared).expect("route parses");
    assert_eq!(view.strategy.as_deref(), Some("weighted"));
    assert_eq!(view.weight, Some(3));
    assert_eq!(view.fallbacks[0].weight, Some(2));
    assert_eq!(view.fallbacks[1].weight, None);
    assert_eq!(view.context_fallbacks.len(), 1);
    assert_eq!(view.context_fallbacks[0].provider, "vertex-long");

    let written = route_to_yaml(&view);
    assert_eq!(written, declared, "a round trip must not change the route");
}

#[test]
fn a_route_without_the_keys_gains_none() {
    let declared: Value = serde_yaml::from_str("model_pattern: m\nprovider: p\n").expect("fixture");
    let view = route_from_yaml(&declared).expect("route parses");
    let written = serde_yaml::to_string(&route_to_yaml(&view)).expect("yaml");
    for key in ["strategy", "weight", "context_fallbacks"] {
        assert!(!written.contains(key), "{key} appeared in {written}");
    }
}

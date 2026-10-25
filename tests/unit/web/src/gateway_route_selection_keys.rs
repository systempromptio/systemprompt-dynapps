//! The console's route view carries core's selection and context-window
//! keys — `strategy`, per-deployment `weight` and `context_fallbacks` — so a
//! console save does not delete them.

use serde_yaml::Value;
use systemprompt_web_admin_core::repositories::gateway_catalog::{route_from_yaml, route_to_yaml};

const ROUTE: &str = r"id: claude-weighted
models:
- claude-*
deployments:
- endpoint: vertex-eu/messages
  weight: 3
- endpoint: vertex-us/messages
  weight: 2
- anthropic/messages
strategy: weighted
context_fallbacks:
- claude-sonnet-5-5-1m
";

#[test]
fn selection_and_context_keys_survive_yaml_view_yaml() {
    let declared: Value = serde_yaml::from_str(ROUTE).expect("fixture");
    let view = route_from_yaml(&declared).expect("route parses");
    assert_eq!(view.strategy.as_deref(), Some("weighted"));
    assert_eq!(view.deployments.len(), 3);
    assert_eq!(view.deployments[0].weight, Some(3));
    assert_eq!(view.deployments[1].weight, Some(2));
    assert_eq!(view.deployments[2].weight, None);
    assert_eq!(view.deployments[2].provider(), "anthropic");
    assert_eq!(
        view.context_fallbacks,
        vec!["claude-sonnet-5-5-1m".to_owned()]
    );

    let written = route_to_yaml(&view).expect("route writes");
    assert_eq!(written, declared, "a round trip must not change the route");
}

#[test]
fn a_route_without_the_keys_gains_none() {
    let declared: Value = serde_yaml::from_str("id: m\nmodels: m\n").expect("fixture");
    let view = route_from_yaml(&declared).expect("route parses");
    let written =
        serde_yaml::to_string(&route_to_yaml(&view).expect("route writes")).expect("yaml");
    for key in ["strategy", "weight", "context_fallbacks", "deployments"] {
        assert!(!written.contains(key), "{key} appeared in {written}");
    }
}

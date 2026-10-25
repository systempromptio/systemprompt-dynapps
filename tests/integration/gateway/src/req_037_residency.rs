//! REQ-037 Data Residency Routing — "Routing rules can enforce that
//! classified workloads remain within the configured jurisdiction/region/
//! sovereign endpoint."
//!
//! A route declaring `requires: { european: true }` refuses to resolve the
//! catalog when any deployment it can dispatch to lacks a `european`
//! governance declaration, and the same `requires:`/`governance:` blocks
//! round-trip from the YAML the operator writes — so the boot-time check the
//! register asks for is enforced by configuration, not by convention.

use systemprompt::manifest::services::catalog::{CatalogError, ProviderConfig};
use systemprompt::manifest::services::{GatewayConfigSpec, ModelGovernance, RouteRequirements};

use crate::support::{Fixture, governance};

const EUROPEAN_ROUTE: &str =
    "  - id: eu-classified\n    models: [\"classified-*\"]\n    requires: {european: true}\n";

fn classified(provider_governance: ModelGovernance) -> Fixture {
    Fixture::default()
        .provider("eu-provider", "openai-chat", provider_governance)
        .model("classified-large", None)
        .deploy("classified-large", &["eu-provider/api"])
        .route(EUROPEAN_ROUTE)
}

#[test]
fn a_european_route_over_a_non_european_provider_fails_validation() {
    match classified(governance(false, false)).resolve() {
        Err(CatalogError::RouteGovernanceUnsatisfied {
            route,
            model,
            endpoint,
            requirements,
        }) => {
            assert_eq!(route.as_str(), "eu-classified");
            assert_eq!(model.as_str(), "classified-large");
            assert_eq!(endpoint, "eu-provider/api");
            assert_eq!(requirements, "european");
        },
        other => panic!("expected RouteGovernanceUnsatisfied, got {other:?}"),
    }
}

#[test]
fn the_same_route_validates_once_the_provider_declares_european_residency() {
    assert!(
        classified(governance(true, false)).resolve().is_ok(),
        "a european provider satisfies the european route requirement"
    );
}

#[test]
fn unmet_names_exactly_the_missing_residency_guarantee() {
    let requires = RouteRequirements {
        european: true,
        no_retain: false,
    };
    let unmet = requires.unmet(governance(false, true));
    assert_eq!(unmet, vec!["european"]);
    assert!(
        requires.unmet(governance(true, false)).is_empty(),
        "a compliant posture leaves nothing unmet"
    );
}

#[test]
fn residency_blocks_round_trip_from_yaml() {
    let spec: GatewayConfigSpec = serde_yaml::from_str(
        r#"
enabled: false
routes:
  - id: eu-classified
    models: ["classified-*"]
    requires:
      european: true
"#,
    )
    .expect("the requires block parses from YAML");
    let requires = spec.routes[0]
        .requires
        .expect("the parsed route carries its requirements");
    assert!(requires.european);
    assert!(!requires.no_retain);

    let provider: ProviderConfig = serde_yaml::from_str(
        r"
credential: eu_key
governance:
  european: true
endpoints:
  api:
    wire: openai-chat
    url: https://eu.example.com/v1
",
    )
    .expect("the governance block parses from YAML");
    assert!(provider.governance.european);
    assert!(!provider.governance.no_retain);
}

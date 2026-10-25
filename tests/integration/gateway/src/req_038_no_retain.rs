//! REQ-038 Provider No-Train / No-Retain Policy Enforcement — "Policy can
//! prevent classified requests from being routed to providers that do not
//! satisfy the configured no-train/no-retain contractual requirement."
//!
//! Mirrors the residency evidence for the `no_retain` flag, and additionally
//! proves the model-level override: a model's own `governance:` block in
//! `models.yaml` beats the provider default on every hop the catalog builds,
//! which is what route validation and dispatch both consult.

use systemprompt::manifest::services::ModelGovernance;
use systemprompt::manifest::services::catalog::CatalogError;

use crate::support::{Fixture, governance};

const NO_RETAIN_ROUTE: &str =
    "  - id: no-retain\n    models: [\"classified-*\"]\n    requires: {no_retain: true}\n";

fn classified(
    provider_governance: ModelGovernance,
    model_governance: Option<ModelGovernance>,
) -> Fixture {
    Fixture::default()
        .provider("retaining-provider", "anthropic", provider_governance)
        .model("classified-fast", model_governance)
        .deploy("classified-fast", &["retaining-provider/api"])
        .route(NO_RETAIN_ROUTE)
}

#[test]
fn a_no_retain_route_over_a_retaining_provider_fails_validation() {
    match classified(ModelGovernance::default(), None).resolve() {
        Err(CatalogError::RouteGovernanceUnsatisfied {
            route,
            model,
            requirements,
            ..
        }) => {
            assert_eq!(route.as_str(), "no-retain");
            assert_eq!(model.as_str(), "classified-fast");
            assert_eq!(requirements, "no_retain");
        },
        other => panic!("expected RouteGovernanceUnsatisfied, got {other:?}"),
    }
}

#[test]
fn the_same_route_validates_once_the_provider_contract_declares_no_retain() {
    assert!(
        classified(governance(false, true), None).resolve().is_ok(),
        "a no-retain provider satisfies the no-retain route requirement"
    );
}

#[test]
fn a_model_level_governance_block_overrides_the_provider_default() {
    let catalog = Fixture::default()
        .provider("mixed-provider", "openai-chat", ModelGovernance::default())
        .model("covered-model", Some(governance(false, true)))
        .model("uncovered-model", None)
        .deploy("covered-model", &["mixed-provider/api"])
        .deploy("uncovered-model", &["mixed-provider/api"])
        .resolve()
        .expect("the catalog resolves");

    assert!(
        catalog.deployments("covered-model")[0].governance.no_retain,
        "the model's own declaration wins over the provider default"
    );
    assert!(
        !catalog.deployments("uncovered-model")[0]
            .governance
            .no_retain,
        "a model without its own block inherits the provider default"
    );
}

#[test]
fn validation_honours_the_model_level_override() {
    assert!(
        classified(ModelGovernance::default(), Some(governance(false, true)))
            .resolve()
            .is_ok(),
        "the only reachable model declares no_retain itself, so the route validates \
         even though the provider default does not"
    );
}

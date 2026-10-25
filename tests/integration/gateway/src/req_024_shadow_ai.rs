//! REQ-024 Shadow AI Detection & Blocking — "Unauthorized or unmanaged AI use
//! can be detected and, where policy requires, blocked or escalated through
//! the governed platform."
//!
//! The catalog is the allowlist: the resolver refuses any model id that is
//! not declared in `models.yaml` (`ResolveError::UnknownModel`), a gateway
//! route glob never selects an endpoint on its own, and a model whose only
//! deployments sit on a disabled provider is refused too. There is no open
//! posture to opt into, so traffic to an un-governed model cannot pass
//! through the platform.

use systemprompt::manifest::services::catalog::{ModelRequest, ResolveError, ResolvedCatalog};

use crate::support::{Fixture, governance};

fn governed_catalog() -> ResolvedCatalog {
    Fixture::default()
        .provider("governed-provider", "anthropic", governance(false, false))
        .disabled_provider("retired-provider", "anthropic")
        .model("governed-model", None)
        .model("retired-model", None)
        .deploy("governed-model", &["governed-provider/api"])
        .deploy("retired-model", &["retired-provider/api"])
        .route("  - id: governed\n    models: [\"governed-*\"]\n")
        .resolve()
        .expect("the catalog resolves")
}

#[test]
fn an_unlisted_model_is_refused() {
    let catalog = governed_catalog();

    assert_eq!(
        catalog
            .resolve(&ModelRequest::new("shadow-model"))
            .expect_err("a model outside the catalog is refused"),
        ResolveError::UnknownModel {
            requested: "shadow-model".to_owned()
        }
    );
}

#[test]
fn a_route_glob_alone_exposes_nothing() {
    let catalog = governed_catalog();

    assert!(
        matches!(
            catalog.resolve(&ModelRequest::new("governed-anything")),
            Err(ResolveError::UnknownModel { .. })
        ),
        "a glob match without a catalog entry is still refused"
    );
}

#[test]
fn a_cataloged_and_deployed_model_is_served() {
    let catalog = governed_catalog();

    let target = catalog
        .resolve(&ModelRequest::new("governed-model"))
        .expect("the catalog model resolves");
    assert_eq!(
        target.primary().expect("a hop").provider.as_str(),
        "governed-provider"
    );
}

#[test]
fn a_model_deployed_only_on_a_disabled_provider_is_refused() {
    let catalog = governed_catalog();

    assert!(
        matches!(
            catalog.resolve(&ModelRequest::new("retired-model")),
            Err(ResolveError::NoEnabledDeployment { .. })
        ),
        "disabling the provider withdraws every model it served"
    );
}

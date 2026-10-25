//! Shared builder for the gateway evidence modules: a catalog document
//! (credentials, providers with endpoints, models, deployments and policy
//! routes) written in the same YAML shape as `services/ai/*.yaml` and
//! resolved through `CatalogDocument::resolve`, the loader's own path. Every
//! provider bills one platform credential; `gateway.enabled` stays false so
//! validation skips the pricing tier, which is billing evidence rather than
//! the routing, residency and exposure evidence these modules are about.

use systemprompt::manifest::services::ModelGovernance;
use systemprompt::manifest::services::catalog::{
    CatalogContext, CatalogDocument, CatalogResult, ResolvedCatalog,
};

#[derive(Debug, Default)]
pub(crate) struct Fixture {
    providers: String,
    models: String,
    deployments: String,
    routes: String,
}

fn governance_yaml(governance: ModelGovernance) -> String {
    format!(
        "{{european: {}, no_retain: {}}}",
        governance.european, governance.no_retain
    )
}

impl Fixture {
    pub(crate) fn provider(mut self, id: &str, wire: &str, governance: ModelGovernance) -> Self {
        self.providers.push_str(&format!(
            "  {id}:\n    credential: platform\n    governance: {}\n    endpoints:\n      \
             api:\n        wire: {wire}\n        url: https://{id}.example.com/v1\n",
            governance_yaml(governance)
        ));
        self
    }

    pub(crate) fn disabled_provider(mut self, id: &str, wire: &str) -> Self {
        self.providers.push_str(&format!(
            "  {id}:\n    credential: platform\n    enabled: false\n    endpoints:\n      \
             api:\n        wire: {wire}\n        url: https://{id}.example.com/v1\n"
        ));
        self
    }

    pub(crate) fn model(mut self, id: &str, governance: Option<ModelGovernance>) -> Self {
        self.models
            .push_str(&format!("  {id}:\n    clients: [claude]\n"));
        if let Some(governance) = governance {
            self.models.push_str(&format!(
                "    governance: {}\n",
                governance_yaml(governance)
            ));
        }
        self
    }

    // Deploys `model` on each `provider/endpoint` (or full deployment
    // mapping) in order: the first is the primary hop.
    pub(crate) fn deploy(mut self, model: &str, targets: &[&str]) -> Self {
        self.deployments.push_str(&format!("  {model}:\n"));
        for target in targets {
            self.deployments.push_str(&format!("  - {target}\n"));
        }
        self
    }

    // One `gateway.routes` entry, written as the YAML list item it is.
    pub(crate) fn route(mut self, yaml: &str) -> Self {
        self.routes.push_str(yaml);
        self
    }

    pub(crate) fn yaml(&self) -> String {
        let routes = if self.routes.is_empty() {
            "  routes: []\n".to_owned()
        } else {
            format!("  routes:\n{}", self.routes)
        };
        format!(
            "credentials:\n  platform:\n    kind: platform\n    secret: test_api_key\n\
             providers:\n{}models:\n{}deployments:\n{}gateway:\n  enabled: false\n{routes}",
            self.providers, self.models, self.deployments
        )
    }

    pub(crate) fn resolve(&self) -> CatalogResult<ResolvedCatalog> {
        CatalogDocument::from_yaml_str("gateway-evidence", &self.yaml())
            .expect("the fixture document parses")
            .resolve(&CatalogContext::for_install("gateway-evidence"))
    }
}

pub(crate) const fn governance(european: bool, no_retain: bool) -> ModelGovernance {
    ModelGovernance {
        european,
        no_retain,
    }
}

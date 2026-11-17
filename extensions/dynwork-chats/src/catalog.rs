//! The models a chat can use, from the instance's catalog.
//!
//! Decision D3: `GET /models` lists what the user has through their
//! systemprompt account, and nothing is hard-coded. Core 0.81 has no
//! per-user model policy (a gateway policy carries quotas, rates and safety
//! scanners; only an API key can carry a model allowlist), so what separates
//! one user's list from another's is the credential behind each model. A reply
//! runs on the in-process `AiService`, which serves platform credentials and
//! leaves member-paid ones (a personal key, the caller's own login) to the
//! gateway, so a model is listed when it is advertised and an enabled
//! deployment of it has a platform credential that is configured: a secret
//! the instance holds, or a workload identity. The provider is the first such
//! deployment, which is the one `AiService` picks for `{ providerID, id }`.
//! Personal-key models join the list when replies go through the gateway
//! (stage 2). [`listed`] takes the catalog and a lookup that says whether the
//! instance holds a secret of a name, and sorts by provider and id.

use systemprompt::manifest::services::catalog::{ResolvedCatalog, ResolvedHop};

/// One model of the list, as `GET /models` answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub reasoning: bool,
    pub tools: bool,
}

pub fn listed(catalog: &ResolvedCatalog, secret_present: impl Fn(&str) -> bool) -> Vec<ModelInfo> {
    let mut models: Vec<ModelInfo> = catalog
        .advertised_models(None)
        .filter_map(|model| {
            let hop = catalog
                .enabled_hops(model.id.as_str())
                .find(|hop| servable(hop, &secret_present))?;
            let provider = hop.provider.as_str();
            let provider_name = catalog
                .provider(provider)
                .and_then(|found| found.display_name.clone())
                .unwrap_or_else(|| provider.to_owned());
            Some(ModelInfo {
                id: model.id.as_str().to_owned(),
                name: model
                    .display_name
                    .clone()
                    .unwrap_or_else(|| model.id.as_str().to_owned()),
                provider_id: provider.to_owned(),
                provider_name,
                reasoning: model.capabilities.reasoning,
                tools: model.capabilities.tools,
            })
        })
        .collect();
    models.sort_by(|a, b| (&a.provider_id, &a.id).cmp(&(&b.provider_id, &b.id)));
    models
}

fn servable(hop: &ResolvedHop, secret_present: &impl Fn(&str) -> bool) -> bool {
    hop.credential.platform_workload_identity().is_some()
        || hop
            .credential
            .platform_secret()
            .is_some_and(|secret| secret_present(secret.as_str()))
}

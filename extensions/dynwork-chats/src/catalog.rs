//! The models a chat can use: what Dynwork Desktop shows the user.
//!
//! Decision D3 and its refinement: `GET /models` is the desktop's
//! own list, by its algorithm, on the same core functions. The desktop reads
//! the user's bridge profile and the catalog and combines them (`gateway()` in
//! its account sync); [`listing`] does what core's `profile` handler and that
//! function do, in process:
//!
//! 1. `bridge_profile::build` gives the models that are advertised and servable
//!    for the instance's secrets, the providers with their `configured` flag,
//!    the limits and the default.
//! 2. `ModelAdmission::retain` drops the models the instance's authz hook
//!    refuses to this caller, asking it the way `/v1/messages` would.
//! 3. Each model takes the first provider of the profile that serves it and is
//!    configured, a name and limits from the profile then the catalog, and the
//!    profile's default is kept when listed.
//!
//! One thing differs from the desktop. A reply runs on the in-process
//! `AiService`, which leaves member-paid credentials (a personal key, the
//! caller's own login) to the gateway, so a provider billed that way is
//! skipped and a model served only by one is left out: a reply to it would
//! fail. For the same reason a member's stored keys change nothing here, and
//! the server passes none; they are an input of [`listing`] so stage 2 (replies
//! through the gateway) can supply them.

use std::collections::BTreeSet;

use systemprompt::api::routes::gateway::bridge_admission::{AdmissionPrincipal, ModelAdmission};
use systemprompt::api::routes::gateway::models::humanize_model_id;
use systemprompt::governance::authz::SharedAuthzHook;
use systemprompt::manifest::bridge_profile::{self, BridgeProfileParams, CredentialFacts};
use systemprompt::manifest::services::catalog::ResolvedCatalog;
use systemprompt::models::bridge::profile::ProviderHealth;

use crate::store::ModelRef;

/// One model of the list, as `GET /models` answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub reasoning: bool,
    pub tools: bool,
    pub context: Option<u32>,
    pub output: Option<u32>,
}

/// What `GET /models` answers: the models and the suggested one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Listing {
    pub models: Vec<ModelInfo>,
    pub default: Option<String>,
}

impl Listing {
    pub fn contains(&self, model: &ModelRef) -> bool {
        let ModelRef {
            provider_id,
            model_id,
        } = model;
        self.models
            .iter()
            .any(|listed| listed.provider_id == *provider_id && listed.id == *model_id)
    }
}

pub async fn listing(
    catalog: &ResolvedCatalog,
    secret_present: &(dyn Fn(&str) -> bool + Sync),
    personal_keys: &BTreeSet<String>,
    hook: &SharedAuthzHook,
    principal: &AdmissionPrincipal,
) -> Listing {
    let mut profile = bridge_profile::build(
        BridgeProfileParams {
            inference_gateway_base_url: String::new(),
            auth_scheme: String::new(),
            organization_uuid: None,
            catalog,
        },
        &CredentialFacts {
            secret_present,
            personal_keys,
            presents_own_credential: false,
        },
    );
    let admission = ModelAdmission {
        catalog,
        hook,
        principal,
    };
    profile.models = admission.retain(std::mem::take(&mut profile.models)).await;
    profile
        .model_limits
        .retain(|model, _| profile.models.contains(model));

    let mut seen = BTreeSet::new();
    let mut models = Vec::new();
    for id in &profile.models {
        if !seen.insert(id.as_str()) {
            continue;
        }
        let Some(model) = catalog.model(id.as_str()) else {
            continue;
        };
        let Some(provider) = profile.providers.iter().find(|provider| {
            provider.configured
                && provider.models.iter().any(|served| served == id.as_str())
                && answers_in_process(catalog, provider)
        }) else {
            continue;
        };
        let limits = profile.model_limits.get(id);
        models.push(ModelInfo {
            id: id.as_str().to_owned(),
            name: model
                .display_name
                .clone()
                .unwrap_or_else(|| humanize_model_id(id.as_str())),
            provider_id: provider.name.clone(),
            provider_name: provider
                .display_name
                .clone()
                .unwrap_or_else(|| provider.name.clone()),
            reasoning: model.capabilities.reasoning,
            tools: model.capabilities.tools,
            context: first_size([
                limits.map(|found| found.context_window),
                Some(model.limits.context_window),
            ]),
            output: first_size([
                limits.map(|found| found.max_output_tokens),
                Some(model.limits.max_output_tokens),
            ]),
        });
    }
    let default = profile
        .default_model
        .as_deref()
        .map(without_context_suffix)
        .filter(|suggested| models.iter().any(|model| model.id == *suggested))
        .map(str::to_owned);
    Listing { models, default }
}

fn answers_in_process(catalog: &ResolvedCatalog, provider: &ProviderHealth) -> bool {
    catalog
        .credential(&provider.credential)
        .is_some_and(|credential| !credential.is_member_paid())
}

fn first_size(sizes: [Option<u32>; 2]) -> Option<u32> {
    sizes.into_iter().flatten().find(|size| *size > 0)
}

fn without_context_suffix(model: &str) -> &str {
    match (model.strip_suffix(']'), model.rfind('[')) {
        (Some(_), Some(open)) => &model[..open],
        _ => model,
    }
}

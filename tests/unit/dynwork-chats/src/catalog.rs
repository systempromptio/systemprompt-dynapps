//! The model list: what Dynwork Desktop shows a user, computed by the same
//! core functions (the bridge profile builder, then model admission through
//! the instance's authz hook) over a catalog, the secrets the instance holds
//! and who is asking.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use dynwork_chats::auth::{Caller, Client};
use dynwork_chats::catalog::{Listing, ModelInfo, listing};
use dynwork_relay::ids::DeviceId;
use systemprompt::api::routes::gateway::bridge_admission::AdmissionPrincipal;
use systemprompt::api::routes::gateway::models::humanize_model_id;
use systemprompt::governance::authz::{
    AuthzDecision, AuthzDecisionHook, AuthzRequest, DenyReason, EntityRef, SharedAuthzHook,
};
use systemprompt::identifiers::{ClientId, SessionId, UserId};
use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::catalog::{CatalogDocument, CatalogInput, ResolvedCatalog};
use systemprompt::manifest::services::{CatalogContext, GatewayConfigSpec};

const CATALOG: &str = r#"
credentials:
  alpha-key: { kind: platform, secret: alpha }
  beta-key: { kind: platform, secret: beta }
  aa-go: { kind: personal, label: Go }
providers:
  alpha:
    credential: alpha-key
    display_name: Alpha Cloud
    endpoints:
      api: { wire: anthropic, url: "https://alpha.invalid/v1" }
  beta:
    credential: beta-key
    endpoints:
      api: { wire: anthropic, url: "https://beta.invalid/v1" }
  aa-go:
    credential: aa-go
    display_name: Go
    endpoints:
      chat: { wire: openai-chat, url: "https://go.invalid/v1" }
models:
  m-big:
    display_name: Big One
    capabilities: { tools: true, reasoning: true }
    limits: { context_window: 1000000, max_output_tokens: 64000 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [claude]
  m-secret:
    capabilities: { tools: true }
    limits: { context_window: 200000, max_output_tokens: 8192 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [claude]
  m-beta:
    limits: { context_window: 100000, max_output_tokens: 4096 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
  m-nolimit:
    limits: { context_window: 0, max_output_tokens: 4096 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
  m-failover:
    limits: { context_window: 128000, max_output_tokens: 4096 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
  m-personal:
    limits: { context_window: 128000, max_output_tokens: 4096 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
  m-hidden:
    limits: { context_window: 128000, max_output_tokens: 4096 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: []
deployments:
  m-big: [alpha/api]
  m-secret: [alpha/api]
  m-beta: [beta/api]
  m-nolimit: [alpha/api]
  m-failover: [beta/api, alpha/api]
  m-personal: [aa-go/chat]
  m-hidden: [alpha/api]
gateway:
  enabled: true
  default_model: m-big
"#;

pub(crate) fn fixture() -> ResolvedCatalog {
    let document = CatalogDocument::from_yaml_str("test", CATALOG).expect("the fixture parses");
    let gateway = document.gateway.clone().map(GatewayConfigSpec::resolve);
    ResolvedCatalog::build(
        CatalogInput {
            credentials: &document.credentials,
            providers: &document.providers,
            models: &document.models,
            deployments: &document.deployments,
            gateway: gateway.as_ref(),
            default_image_model: None,
        },
        &CatalogContext::default(),
    )
    .expect("the fixture resolves")
}

// Denies a model to the roles it names, as an access rule would.
#[derive(Debug, Default)]
struct RoleHook {
    denied: Vec<(&'static str, &'static str)>,
    seen: Mutex<Vec<AuthzRequest>>,
}

impl RoleHook {
    fn denying(model: &'static str, role: &'static str) -> Arc<Self> {
        Arc::new(Self {
            denied: vec![(model, role)],
            ..Self::default()
        })
    }

    fn requests(&self) -> Vec<AuthzRequest> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn decide(&self, req: &AuthzRequest) -> AuthzDecision {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(req.clone());
        let EntityRef::Model(model) = &req.entity else {
            return AuthzDecision::Allow;
        };
        let refused = self.denied.iter().find(|(denied_model, role)| {
            *denied_model == model.as_str() && req.roles.iter().any(|have| have == role)
        });
        match refused {
            Some((_, role)) => AuthzDecision::Deny {
                reason: DenyReason::RoleDeny {
                    entity: req.entity.clone(),
                    role: (*role).to_owned(),
                    justification: None,
                },
                policy: "test".to_owned(),
            },
            None => AuthzDecision::Allow,
        }
    }
}

#[async_trait]
impl AuthzDecisionHook for RoleHook {
    async fn evaluate(&self, req: AuthzRequest) -> AuthzDecision {
        self.decide(&req)
    }

    async fn probe(&self, req: AuthzRequest) -> AuthzDecision {
        self.decide(&req)
    }
}

fn principal(user: &str, roles: &[&str]) -> AdmissionPrincipal {
    AdmissionPrincipal {
        user_id: UserId::new(user),
        roles: roles.iter().map(|role| (*role).to_owned()).collect(),
        client_id: None,
        session_id: SessionId::generate(),
    }
}

async fn listed(
    catalog: &ResolvedCatalog,
    secrets: &[&str],
    keys: &[&str],
    hook: &SharedAuthzHook,
    who: &AdmissionPrincipal,
) -> Listing {
    let held: BTreeSet<String> = secrets.iter().map(|name| (*name).to_owned()).collect();
    let personal: BTreeSet<String> = keys.iter().map(|name| (*name).to_owned()).collect();
    listing(catalog, &|name| held.contains(name), &personal, hook, who).await
}

fn ids(models: &[ModelInfo]) -> Vec<&str> {
    models.iter().map(|model| model.id.as_str()).collect()
}

fn find<'a>(models: &'a [ModelInfo], id: &str) -> &'a ModelInfo {
    models
        .iter()
        .find(|model| model.id == id)
        .unwrap_or_else(|| panic!("{id} is not listed: {:?}", ids(models)))
}

fn allow_all() -> SharedAuthzHook {
    RoleHook::denying("none", "none")
}

#[tokio::test]
async fn a_role_the_hook_denies_loses_the_model() {
    let catalog = fixture();
    let hook: SharedAuthzHook = RoleHook::denying("m-secret", "intern");
    let staff = listed(&catalog, &["alpha"], &[], &hook, &principal("u1", &["staff"])).await;
    let intern = listed(&catalog, &["alpha"], &[], &hook, &principal("u2", &["intern"])).await;
    assert!(ids(&staff.models).contains(&"m-secret"));
    assert!(!ids(&intern.models).contains(&"m-secret"));
    assert!(ids(&intern.models).contains(&"m-big"));
}

#[tokio::test]
async fn two_users_with_different_roles_get_different_lists() {
    let catalog = fixture();
    let hook: SharedAuthzHook = RoleHook::denying("m-big", "intern");
    let staff = listed(&catalog, &["alpha"], &[], &hook, &principal("u1", &["staff"])).await;
    let intern = listed(&catalog, &["alpha"], &[], &hook, &principal("u2", &["intern"])).await;
    assert_ne!(ids(&staff.models), ids(&intern.models));
    assert_eq!(staff.models.len(), intern.models.len() + 1);
}

#[tokio::test]
async fn the_hook_is_asked_about_the_caller_it_is_listing_for() {
    let catalog = fixture();
    let recorder = RoleHook::denying("none", "none");
    let hook: SharedAuthzHook = Arc::<RoleHook>::clone(&recorder);
    let who = AdmissionPrincipal {
        user_id: UserId::new("u1"),
        roles: vec!["staff".to_owned()],
        client_id: Some(ClientId::new("sp_bridge")),
        session_id: SessionId::new("sess_fixed"),
    };
    listed(&catalog, &["alpha"], &[], &hook, &who).await;
    let asked = recorder.requests();
    assert!(!asked.is_empty());
    for request in asked {
        assert_eq!(request.user_id, who.user_id);
        assert_eq!(request.roles, who.roles);
        assert_eq!(request.client_id, who.client_id);
        assert_eq!(request.session_id.as_ref(), Some(&who.session_id));
    }
}

#[tokio::test]
async fn an_unconfigured_provider_drops_its_model() {
    let catalog = fixture();
    let hook = allow_all();
    let who = principal("u1", &["staff"]);
    let alone = listed(&catalog, &["alpha"], &[], &hook, &who).await;
    assert!(!ids(&alone.models).contains(&"m-beta"));
    let both = listed(&catalog, &["alpha", "beta"], &[], &hook, &who).await;
    let beta = find(&both.models, "m-beta");
    assert_eq!(beta.provider_id, "beta");
    assert_eq!(beta.provider_name, "beta");
}

#[tokio::test]
async fn name_and_limits_come_from_the_profile_then_the_catalog() {
    let catalog = fixture();
    let hook = allow_all();
    let models = listed(&catalog, &["alpha"], &[], &hook, &principal("u1", &[]))
        .await
        .models;
    let big = find(&models, "m-big");
    assert_eq!(big.name, "Big One");
    assert_eq!((big.context, big.output), (Some(1_000_000), Some(64_000)));
    assert_eq!(big.provider_id, "alpha");
    assert_eq!(big.provider_name, "Alpha Cloud");
    assert!(big.reasoning && big.tools);
    let plain = find(&models, "m-secret");
    assert_eq!(plain.name, humanize_model_id("m-secret"));
    assert_eq!(plain.context, Some(200_000));
    assert!(!plain.reasoning && plain.tools);
    let nolimit = find(&models, "m-nolimit");
    assert_eq!((nolimit.context, nolimit.output), (None, Some(4096)));
}

#[tokio::test]
async fn the_default_is_the_catalogs_without_its_context_suffix_and_only_when_listed() {
    let catalog = fixture();
    let who = principal("u1", &["intern"]);
    let open = listed(&catalog, &["alpha"], &[], &allow_all(), &who).await;
    assert_eq!(open.default.as_deref(), Some("m-big"));
    let hook: SharedAuthzHook = RoleHook::denying("m-big", "intern");
    let denied = listed(&catalog, &["alpha"], &[], &hook, &who).await;
    assert_eq!(denied.default, None);
    let no_secret = listed(&catalog, &[], &[], &allow_all(), &who).await;
    assert_eq!(no_secret.default, None);
}

#[tokio::test]
async fn a_model_not_advertised_stays_out_whatever_the_hook_says() {
    let catalog = fixture();
    let models = listed(&catalog, &["alpha"], &[], &allow_all(), &principal("u1", &[])).await;
    assert!(!ids(&models.models).contains(&"m-hidden"));
}

#[tokio::test]
async fn a_members_own_key_adds_no_model_the_instance_cannot_answer_with() {
    let catalog = fixture();
    let hook = allow_all();
    let who = principal("u1", &[]);
    let with_key = listed(&catalog, &["alpha"], &["aa-go"], &hook, &who).await;
    let without = listed(&catalog, &["alpha"], &[], &hook, &who).await;
    assert_eq!(with_key, without);
    assert!(!ids(&with_key.models).contains(&"m-personal"));
}

#[tokio::test]
async fn a_model_with_two_platform_providers_names_the_first_configured_one_in_the_profile_order() {
    let catalog = fixture();
    let hook = allow_all();
    let who = principal("u1", &[]);
    let both = listed(&catalog, &["alpha", "beta"], &[], &hook, &who).await.models;
    assert_eq!(find(&both, "m-failover").provider_id, "alpha");
    let only_beta = listed(&catalog, &["beta"], &[], &hook, &who).await.models;
    assert_eq!(find(&only_beta, "m-failover").provider_id, "beta");
}

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("services/ai/models.yaml").is_file() {
            return dir;
        }
        assert!(dir.pop(), "repository root not found");
    }
}

fn committed() -> ResolvedCatalog {
    ConfigLoader::load_from_path(&repo_root().join("services/config/config.yaml"), None)
        .expect("the services tree loads")
        .resolve_catalog(&CatalogContext::default())
        .expect("the catalog resolves")
}

#[tokio::test]
async fn the_committed_catalog_follows_the_secrets_the_instance_holds() {
    let catalog = committed();
    let hook = allow_all();
    let who = principal("u1", &[]);
    let both = listed(&catalog, &["anthropic", "deepseek"], &[], &hook, &who).await;
    let claude = listed(&catalog, &["anthropic"], &[], &hook, &who).await;
    let deepseek = listed(&catalog, &["deepseek"], &[], &hook, &who).await;
    assert!(!claude.models.is_empty() && !deepseek.models.is_empty());
    assert!(claude.models.iter().all(|model| model.provider_id == "anthropic"));
    assert!(deepseek.models.iter().all(|model| model.provider_id == "deepseek"));
    assert_eq!(both.models.len(), claude.models.len() + deepseek.models.len());
    let nothing = listed(&catalog, &[], &[], &hook, &who).await;
    assert!(nothing.models.is_empty());
}

#[tokio::test]
async fn the_committed_catalogs_member_paid_models_are_not_listed_even_with_a_key() {
    let catalog = committed();
    let personal: Vec<&str> = catalog
        .advertised_models(None)
        .filter(|model| {
            catalog
                .enabled_hops(model.id.as_str())
                .all(|hop| hop.credential.is_member_paid())
        })
        .map(|model| model.id.as_str())
        .collect();
    assert!(!personal.is_empty(), "the catalog advertises member-paid models");
    let keys: Vec<&str> = catalog
        .credentials()
        .filter(|credential| credential.is_personal())
        .map(|credential| credential.id.as_str())
        .collect();
    let models = listed(
        &catalog,
        &["anthropic", "deepseek", "opencode-go"],
        &keys,
        &allow_all(),
        &principal("u1", &[]),
    )
    .await
    .models;
    for id in personal {
        assert!(!ids(&models).contains(&id), "{id} needs the gateway");
    }
}

#[tokio::test]
async fn a_listed_model_names_a_provider_that_serves_it() {
    let catalog = committed();
    let models = listed(
        &catalog,
        &["anthropic", "deepseek"],
        &[],
        &allow_all(),
        &principal("u1", &[]),
    )
    .await
    .models;
    for model in models {
        let served = catalog
            .enabled_hops(&model.id)
            .any(|hop| hop.provider.as_str() == model.provider_id);
        assert!(served, "{}", model.id);
        assert!(!model.name.is_empty());
        assert!(!model.provider_name.is_empty());
        assert!(model.tools, "the catalog marks these models as tool-capable");
        assert!(model.context.is_some() && model.output.is_some());
    }
}

fn caller(client: Client, session: Option<SessionId>) -> Caller {
    Caller {
        user: UserId::new("u1"),
        client,
        peer: None,
        client_id: None,
        session,
    }
}

#[test]
fn a_phone_presents_one_stable_session_of_its_own() {
    let device = DeviceId::generate();
    let first = caller(Client::Phone(device), None).admission_session();
    assert_eq!(first, caller(Client::Phone(device), None).admission_session());
    assert_ne!(
        first,
        caller(Client::Phone(DeviceId::generate()), None).admission_session()
    );
    assert!(first.as_str().starts_with("sess_"));
}

#[test]
fn a_desktop_presents_its_tokens_session_else_one_derived_from_the_user() {
    let token_session = SessionId::new("sess_from-the-token");
    assert_eq!(
        caller(Client::Desktop, Some(token_session.clone())).admission_session(),
        token_session
    );
    let derived = caller(Client::Desktop, None).admission_session();
    assert_eq!(derived, caller(Client::Desktop, None).admission_session());
    assert_ne!(derived, caller(Client::Phone(DeviceId::generate()), None).admission_session());
}

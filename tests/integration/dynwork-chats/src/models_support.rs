//! A small catalog and an authz hook that denies a model to a role, standing
//! in for the instance's catalog and access rules behind `GET /models`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use dynwork_chats::error::ChatsError;
use dynwork_chats::roles::Roles;
use systemprompt::governance::authz::{
    AuthzDecision, AuthzDecisionHook, AuthzRequest, DenyReason, EntityRef, SharedAuthzHook,
};
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::catalog::{CatalogDocument, CatalogInput, ResolvedCatalog};
use systemprompt::manifest::services::{CatalogContext, GatewayConfigSpec};

pub(crate) const DENIED_MODEL: &str = "claude-opus";
pub(crate) const DENIED_ROLE: &str = "intern";

const CATALOG: &str = r#"
credentials:
  anthropic: { kind: platform, secret: anthropic }
  deepseek: { kind: platform, secret: deepseek }
  go: { kind: personal, label: Go }
providers:
  anthropic:
    credential: anthropic
    display_name: Anthropic
    endpoints:
      api: { wire: anthropic, url: "https://anthropic.invalid/v1" }
  deepseek:
    credential: deepseek
    endpoints:
      api: { wire: anthropic, url: "https://deepseek.invalid/v1" }
  go:
    credential: go
    endpoints:
      chat: { wire: openai-chat, url: "https://go.invalid/v1" }
models:
  claude-opus:
    display_name: Claude Opus
    capabilities: { tools: true, reasoning: true }
    limits: { context_window: 1000000, max_output_tokens: 128000 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [claude]
  claude-sonnet:
    display_name: Claude Sonnet
    capabilities: { tools: true, reasoning: true }
    limits: { context_window: 200000, max_output_tokens: 64000 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [claude]
  deepseek-chat:
    display_name: deepseek-chat
    limits: { context_window: 128000, max_output_tokens: 8192 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
  go-kimi:
    limits: { context_window: 128000, max_output_tokens: 8192 }
    pricing: { input_per_million: 1.0, output_per_million: 2.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0 }
    clients: [openai]
deployments:
  claude-opus: [anthropic/api]
  claude-sonnet: [anthropic/api]
  deepseek-chat: [deepseek/api]
  go-kimi: [go/chat]
gateway:
  enabled: true
  default_model: claude-sonnet
"#;

pub(crate) fn catalog() -> ResolvedCatalog {
    let document = CatalogDocument::from_yaml_str("test", CATALOG).expect("the catalog parses");
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
    .expect("the catalog resolves")
}

#[derive(Debug)]
struct RoleHook;

impl RoleHook {
    fn decide(req: &AuthzRequest) -> AuthzDecision {
        let denied = matches!(&req.entity, EntityRef::Model(model) if model.as_str() == DENIED_MODEL)
            && req.roles.iter().any(|role| role == DENIED_ROLE);
        if denied {
            return AuthzDecision::Deny {
                reason: DenyReason::RoleDeny {
                    entity: req.entity.clone(),
                    role: DENIED_ROLE.to_owned(),
                    justification: None,
                },
                policy: "test".to_owned(),
            };
        }
        // The rule for the OAuth client of the bridge: it may not use deepseek.
        let client_denied = req
            .client_id
            .as_ref()
            .is_some_and(|client| client.as_str() == "sp_nodeepseek")
            && matches!(&req.entity, EntityRef::Model(model) if model.as_str() == "deepseek-chat");
        if client_denied {
            return AuthzDecision::Deny {
                reason: DenyReason::RoleDeny {
                    entity: req.entity.clone(),
                    role: "client".to_owned(),
                    justification: None,
                },
                policy: "test".to_owned(),
            };
        }
        AuthzDecision::Allow
    }
}

#[async_trait]
impl AuthzDecisionHook for RoleHook {
    async fn evaluate(&self, req: AuthzRequest) -> AuthzDecision {
        Self::decide(&req)
    }

    async fn probe(&self, req: AuthzRequest) -> AuthzDecision {
        Self::decide(&req)
    }
}

pub(crate) fn role_hook() -> SharedAuthzHook {
    Arc::new(RoleHook)
}

// The roles of the instance's users, set by the tests; a user never set has
// none.
#[derive(Debug, Default)]
pub(crate) struct FakeRoles(Mutex<HashMap<String, Vec<String>>>);

impl FakeRoles {
    pub(crate) fn set(&self, user: &str, roles: &[&str]) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                user.to_owned(),
                roles.iter().map(|role| (*role).to_owned()).collect(),
            );
    }
}

#[async_trait]
impl Roles for FakeRoles {
    async fn of(&self, user: &UserId) -> Result<Option<Vec<String>>, ChatsError> {
        let held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(Some(held.get(user.as_str()).cloned().unwrap_or_default()))
    }
}

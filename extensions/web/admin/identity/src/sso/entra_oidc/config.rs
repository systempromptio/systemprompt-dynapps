//! Configuration of the Microsoft Entra ID `OpenID Connect` relying party
//! (`services/web/config/entra-oidc.yaml`).
//!
//! The aggregator loads the file at boot and hands it to [`install_config`]
//! before the SSO driver discovers providers; [`super::factory`] reads it back
//! from the `OnceLock`. The absolute `redirect_uri` is never authored in YAML:
//! the loader derives it from the profile's `api_external_url`, so one
//! committed file is correct on every deployment host.

use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use systemprompt::sso::SsoIdentityPolicy;

pub const ID: &str = "entra-oidc";

const ENTRA_AUTHORITY: &str = "https://login.microsoftonline.com";

static CONFIG: OnceLock<Arc<EntraOidcConfig>> = OnceLock::new();

fn default_client_secret_secret() -> String {
    "entra_oidc_client_secret".to_owned()
}

fn default_redirect_path() -> String {
    "/admin/auth/entra/callback".to_owned()
}

fn default_scopes() -> Vec<String> {
    ["openid", "profile", "email"].map(str::to_owned).to_vec()
}

fn default_groups_claim() -> String {
    "groups".to_owned()
}

const fn default_clock_skew_seconds() -> u64 {
    120
}

fn default_jwks_allowed_hosts() -> Vec<String> {
    vec!["login.microsoftonline.com".to_owned()]
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntraOidcConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, rename = "tenant_id")]
    pub entra_tenant_id: String,
    #[serde(default, rename = "client_id")]
    pub entra_client_id: String,
    #[serde(default = "default_client_secret_secret")]
    pub client_secret_secret: String,
    #[serde(default = "default_redirect_path")]
    pub redirect_path: String,
    #[serde(skip)]
    pub redirect_uri: Option<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    // Why: tests and sovereign clouds override Entra ID's well-known discovery
    // URL; the discovery document's `issuer` is then authoritative.
    #[serde(default)]
    pub discovery_url: Option<String>,
    #[serde(default = "default_jwks_allowed_hosts")]
    pub jwks_allowed_hosts: Vec<String>,
    // Why: Entra ID marks a guest or external-IdP `email` unverified; this
    // switch trusts it anyway, so such a user links to an account by email.
    #[serde(default)]
    pub trust_guest_email: bool,
    #[serde(flatten)]
    pub policy: SsoIdentityPolicy,
}

impl Default for EntraOidcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            entra_tenant_id: String::new(),
            entra_client_id: String::new(),
            client_secret_secret: default_client_secret_secret(),
            redirect_path: default_redirect_path(),
            redirect_uri: None,
            scopes: default_scopes(),
            groups_claim: default_groups_claim(),
            clock_skew_seconds: default_clock_skew_seconds(),
            discovery_url: None,
            jwks_allowed_hosts: default_jwks_allowed_hosts(),
            trust_guest_email: false,
            policy: SsoIdentityPolicy::default(),
        }
    }
}

impl EntraOidcConfig {
    #[must_use]
    pub fn issuer(&self) -> String {
        format!("{ENTRA_AUTHORITY}/{}/v2.0", self.entra_tenant_id)
    }

    #[must_use]
    pub fn discovery_url(&self) -> String {
        self.discovery_url
            .clone()
            .unwrap_or_else(|| format!("{}/.well-known/openid-configuration", self.issuer()))
    }

    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.enabled
            && !self.entra_tenant_id.trim().is_empty()
            && !self.entra_client_id.trim().is_empty()
            && self
                .redirect_uri
                .as_deref()
                .is_some_and(|uri| !uri.is_empty())
    }
}

// Why: core's SSO driver discovers providers once, so the first install wins
// and a later one is only warned about.
pub fn install_config(cfg: Arc<EntraOidcConfig>) {
    if CONFIG.set(cfg).is_err() {
        tracing::warn!("Entra OIDC config installed twice; keeping the first");
    }
}

pub(super) fn installed_config() -> Option<Arc<EntraOidcConfig>> {
    CONFIG.get().cloned()
}

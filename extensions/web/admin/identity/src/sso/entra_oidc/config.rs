//! Configuration of the Microsoft Entra ID OpenID Connect relying party
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
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub client_id: String,
    /// Key in the secrets store holding the client secret; never the secret.
    #[serde(default = "default_client_secret_secret")]
    pub client_secret_secret: String,
    #[serde(default = "default_redirect_path")]
    pub redirect_path: String,
    /// Filled by the loader from `api_external_url` + `redirect_path`.
    #[serde(skip)]
    pub redirect_uri: Option<String>,
    #[serde(default = "default_scopes")]
    pub scopes: Vec<String>,
    #[serde(default = "default_groups_claim")]
    pub groups_claim: String,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    /// Overrides the tenant's well-known discovery document (tests, sovereign
    /// clouds). When set, the discovery document's `issuer` is authoritative.
    #[serde(default)]
    pub discovery_url: Option<String>,
    #[serde(default = "default_jwks_allowed_hosts")]
    pub jwks_allowed_hosts: Vec<String>,
    #[serde(flatten)]
    pub policy: SsoIdentityPolicy,
}

impl Default for EntraOidcConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tenant_id: String::new(),
            client_id: String::new(),
            client_secret_secret: default_client_secret_secret(),
            redirect_path: default_redirect_path(),
            redirect_uri: None,
            scopes: default_scopes(),
            groups_claim: default_groups_claim(),
            clock_skew_seconds: default_clock_skew_seconds(),
            discovery_url: None,
            jwks_allowed_hosts: default_jwks_allowed_hosts(),
            policy: SsoIdentityPolicy::default(),
        }
    }
}

impl EntraOidcConfig {
    /// The v2.0 issuer Entra ID stamps on this tenant's id_tokens.
    #[must_use]
    pub fn issuer(&self) -> String {
        format!("{ENTRA_AUTHORITY}/{}/v2.0", self.tenant_id)
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
            && !self.tenant_id.trim().is_empty()
            && !self.client_id.trim().is_empty()
            && self
                .redirect_uri
                .as_deref()
                .is_some_and(|uri| !uri.is_empty())
    }
}

/// Stores the loaded config for [`super::factory`]. The first call wins; a
/// second is ignored with a warning, since the driver has already discovered
/// providers by then.
pub fn install_config(cfg: Arc<EntraOidcConfig>) {
    if CONFIG.set(cfg).is_err() {
        tracing::warn!("Entra OIDC config installed twice; keeping the first");
    }
}

pub(super) fn installed_config() -> Option<Arc<EntraOidcConfig>> {
    CONFIG.get().cloned()
}

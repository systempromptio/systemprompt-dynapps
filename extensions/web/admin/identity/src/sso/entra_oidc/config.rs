//! Configuration of the Microsoft Entra ID `OpenID Connect` relying party
//! (`services/web/config/entra-oidc.yaml`).
//!
//! [`super::factory`] loads the file from the active services root when
//! core's SSO driver discovers providers, so the config is owned by the
//! provider it builds. The absolute `redirect_uri` is never authored in YAML:
//! the loader derives it from the profile's `api_external_url`, so one
//! committed file is correct on every deployment host. A missing file is a
//! disabled provider, never an error.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use systemprompt::sso::SsoIdentityPolicy;

pub const ID: &str = "entra-oidc";
pub const CONFIG_FILE: &str = "web/config/entra-oidc.yaml";

const ENTRA_AUTHORITY: &str = "https://login.microsoftonline.com";

#[derive(Debug, thiserror::Error)]
pub enum EntraOidcConfigError {
    #[error("{path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    // installation trusts it so such users link by email.
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

    pub fn load(
        services_root: &Path,
        api_external_url: &str,
    ) -> Result<Self, EntraOidcConfigError> {
        let path = services_root.join(CONFIG_FILE);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => return Err(EntraOidcConfigError::Read { path, source }),
        };
        let mut config: Self =
            serde_yaml::from_str(&text).map_err(|source| EntraOidcConfigError::Parse {
                path: path.clone(),
                source,
            })?;
        config.derive_redirect_uri(api_external_url);
        tracing::info!(
            enabled = config.enabled,
            usable = config.is_usable(),
            "Loaded Entra OIDC config"
        );
        Ok(config)
    }

    // Why: Microsoft Entra ID answers `AADSTS50011` when the redirect URI is
    // not one registered on the App Registration, so it derives from this
    // host's own `api_external_url`.
    fn derive_redirect_uri(&mut self, api_external_url: &str) {
        if self.redirect_uri.is_some() {
            return;
        }
        let base = api_external_url.trim_end_matches('/');
        if base.is_empty() {
            return;
        }
        self.redirect_uri = Some(format!("{base}{}", self.redirect_path));
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

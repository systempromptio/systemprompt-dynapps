//! SSO connection config, loaded from `services/web/config/adfs.yaml`:
//! a SAML 2.0 relying party trusted by a Microsoft Entra ID enterprise
//! application.
//!
//! The identity half — allowed domains, group→role map, `auto_provision`,
//! `deny_without_group` — is core's [`SsoIdentityPolicy`], flattened so the
//! YAML keeps its historical shape; the driver applies it. What stays here is
//! the SAML wire: the two URLs, the federation metadata, and the claim URIs.

use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use systemprompt::sso::SsoIdentityPolicy;

// Why: Microsoft Entra ID's default claim URIs for a SAML enterprise
// application; an application whose claim mapping renames them overrides them.
fn default_email_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".to_owned()
}

fn default_name_attribute() -> String {
    "http://schemas.microsoft.com/identity/claims/displayname".to_owned()
}

fn default_groups_attribute() -> String {
    "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups".to_owned()
}

// Why: Microsoft Entra ID replaces the groups claim with this link when a user
// belongs to more groups than a SAML token may carry (150).
pub const GROUPS_OVERAGE_ATTRIBUTE: &str = "http://schemas.microsoft.com/claims/groups.link";

const fn default_allow_idp_initiated() -> bool {
    true
}

const fn default_clock_skew_seconds() -> u64 {
    120
}

// Why: the Entra ID enterprise application registers these URLs as its
// Identifier and Reply URL character for character; the loader appends them
// to the profile's external URL so trust and requests agree.
pub const METADATA_PATH: &str = "/saml/metadata";
pub const ACS_PATH: &str = "/admin/auth/adfs/acs";
pub const START_PATH: &str = "/admin/auth/adfs/start";

/// SSO connection config (SAML 2.0 relying party of Microsoft Entra ID).
///
/// There is no client secret: trust runs on the `IdP` signing certificate in
/// its federation metadata. `idp_metadata_url` is fetched at boot and every
/// six hours, so a certificate roll-over needs no redeploy;
/// `idp_metadata_path` (relative to `services/web/config/`) is a committed
/// copy used when no URL is set or the fetch fails. `entity_id` is our
/// `<Audience>`/`<Issuer>` and `acs_url` must equal the application's
/// registered Reply URL; both are derived from the profile's external URL
/// when omitted. `idp_entity_id` pins the issuer federated identities are
/// keyed by; otherwise it is read from the pinned metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntraSamlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub acs_url: String,
    #[serde(default)]
    pub idp_metadata_path: String,
    #[serde(default)]
    pub idp_metadata_url: Option<String>,
    #[serde(default)]
    pub idp_entity_id: Option<String>,
    #[serde(skip)]
    pub idp_metadata_xml: String,
    #[serde(default = "default_email_attribute")]
    pub email_attribute: String,
    #[serde(default = "default_name_attribute")]
    pub name_attribute: String,
    #[serde(default = "default_groups_attribute")]
    pub groups_attribute: String,
    #[serde(default = "default_allow_idp_initiated")]
    pub allow_idp_initiated: bool,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    #[serde(flatten)]
    pub policy: SsoIdentityPolicy,
}

impl EntraSamlConfig {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            entity_id: String::new(),
            acs_url: String::new(),
            idp_metadata_path: String::new(),
            idp_metadata_url: None,
            idp_entity_id: None,
            idp_metadata_xml: String::new(),
            email_attribute: default_email_attribute(),
            name_attribute: default_name_attribute(),
            groups_attribute: default_groups_attribute(),
            allow_idp_initiated: default_allow_idp_initiated(),
            clock_skew_seconds: default_clock_skew_seconds(),
            policy: SsoIdentityPolicy::default(),
        }
    }

    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled
            && !self.entity_id.is_empty()
            && !self.acs_url.is_empty()
            && (!self.idp_metadata_xml.is_empty() || self.idp_metadata_url.is_some())
    }
}

static CONFIG: OnceLock<Arc<EntraSamlConfig>> = OnceLock::new();

// Why: core's SSO driver discovers providers once; this must run before it,
// and the first call wins.
pub fn install_config(cfg: Arc<EntraSamlConfig>) {
    if CONFIG.set(cfg).is_err() {
        tracing::warn!("Entra SAML config installed twice; keeping the first");
    }
}

pub(super) fn installed_config() -> Option<Arc<EntraSamlConfig>> {
    CONFIG.get().cloned()
}

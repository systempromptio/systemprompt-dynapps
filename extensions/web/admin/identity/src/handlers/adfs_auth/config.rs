//! SSO connection config, loaded from `services/web/config/adfs.yaml`:
//! a SAML 2.0 relying party trusted by a Microsoft Entra ID enterprise
//! application.
//!
//! SSO is the only way a non-operator account comes into existence: the
//! domain allow-list and the Entra ID group map are the two halves of the
//! provisioning gate. Operators are created out-of-band with
//! `admin users create`. `auto_provision` defaults off — an allow-listed domain
//! says an address *could* belong to someone, only a mapped group says IT
//! approved them — so SSO signs in existing accounts unless an installation
//! trusts its group map.

use std::collections::BTreeMap;

use serde::Deserialize;

// Why: Microsoft Entra ID's default claim URIs for a SAML enterprise
// application; an application whose claim mapping renames them overrides them.
pub(super) fn default_email_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".to_owned()
}

pub(super) fn default_name_attribute() -> String {
    "http://schemas.microsoft.com/identity/claims/displayname".to_owned()
}

pub(super) fn default_groups_attribute() -> String {
    "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups".to_owned()
}

// Why: Microsoft Entra ID replaces the groups claim with this link when a user
// belongs to more groups than a SAML token may carry (150).
pub const GROUPS_OVERAGE_ATTRIBUTE: &str = "http://schemas.microsoft.com/claims/groups.link";

pub(super) fn default_allowed_domains() -> Vec<String> {
    vec!["dynapps.es".to_owned(), "wearedynapps.com".to_owned()]
}

pub(super) const fn default_auto_provision() -> bool {
    false
}

pub(super) const fn default_deny_without_group() -> bool {
    true
}

pub(super) const fn default_allow_idp_initiated() -> bool {
    true
}

pub(super) const fn default_clock_skew_seconds() -> u64 {
    120
}

// Why: the Entra ID enterprise application registers these URLs as its
// Identifier and Reply URL character for character;
// the loader appends them to the profile's external URL so trust and requests
// agree.
pub const METADATA_PATH: &str = "/saml/metadata";
pub const ACS_PATH: &str = "/admin/auth/adfs/acs";

/// SSO connection config (SAML 2.0 relying party of Microsoft Entra ID).
///
/// There is no client secret: trust runs on the `IdP` signing certificate in
/// its federation metadata. `idp_metadata_url` (the enterprise application's
/// "App Federation Metadata Url") is fetched at boot and every six hours, so
/// a certificate roll-over needs no redeploy; `idp_metadata_path` (relative to
/// `services/web/config/`) is a committed copy used when no URL is set or the
/// fetch fails. At least one of the two is required when `enabled`.
/// `entity_id` is our `<Audience>`/`<Issuer>` and `acs_url` must equal the
/// application's registered Reply URL; both are derived from the
/// profile's external URL when omitted. `deny_without_group` refuses an
/// assertion with no group at all; `group_roles` (exact) and
/// `group_role_patterns` (one-`*` globs, additive) are re-projected onto the
/// directory half of `users.roles` on every login, so hand-granted roles
/// survive.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdfsConfig {
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
    #[serde(skip)]
    pub idp_metadata_xml: String,
    #[serde(default = "default_allowed_domains")]
    pub allowed_email_domains: Vec<String>,
    #[serde(default = "default_auto_provision")]
    pub auto_provision: bool,
    #[serde(default = "default_email_attribute")]
    pub email_attribute: String,
    #[serde(default = "default_name_attribute")]
    pub name_attribute: String,
    #[serde(default = "default_groups_attribute")]
    pub groups_attribute: String,
    #[serde(default = "default_deny_without_group")]
    pub deny_without_group: bool,
    #[serde(default = "default_allow_idp_initiated")]
    pub allow_idp_initiated: bool,
    #[serde(default = "default_clock_skew_seconds")]
    pub clock_skew_seconds: u64,
    #[serde(default)]
    pub group_roles: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub group_role_patterns: BTreeMap<String, Vec<String>>,
}

#[must_use]
pub fn group_matches_pattern(pattern: &str, group: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == group,
        Some((prefix, suffix)) => {
            group.len() >= prefix.len() + suffix.len()
                && group.starts_with(prefix)
                && group.ends_with(suffix)
        },
    }
}

impl AdfsConfig {
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            entity_id: String::new(),
            acs_url: String::new(),
            idp_metadata_path: String::new(),
            idp_metadata_url: None,
            idp_metadata_xml: String::new(),
            allowed_email_domains: default_allowed_domains(),
            auto_provision: default_auto_provision(),
            email_attribute: default_email_attribute(),
            name_attribute: default_name_attribute(),
            groups_attribute: default_groups_attribute(),
            deny_without_group: default_deny_without_group(),
            allow_idp_initiated: default_allow_idp_initiated(),
            clock_skew_seconds: default_clock_skew_seconds(),
            group_roles: BTreeMap::new(),
            group_role_patterns: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled
            && !self.entity_id.is_empty()
            && !self.acs_url.is_empty()
            && (!self.idp_metadata_xml.is_empty() || self.idp_metadata_url.is_some())
    }

    #[must_use]
    pub fn email_allowed(&self, email: &str) -> bool {
        email
            .rsplit('@')
            .next()
            .is_some_and(|domain| self.allowed_email_domains.iter().any(|d| d == domain))
    }

    #[must_use]
    pub fn roles_for_groups(&self, groups: &[String]) -> Vec<String> {
        let mut roles: Vec<String> = groups
            .iter()
            .filter_map(|g| self.group_roles.get(g))
            .flatten()
            .cloned()
            .collect();
        roles.extend(
            self.group_role_patterns
                .iter()
                .filter(|(pattern, _)| groups.iter().any(|g| group_matches_pattern(pattern, g)))
                .flat_map(|(_, granted)| granted.iter().cloned()),
        );
        roles.sort();
        roles.dedup();
        roles
    }
}

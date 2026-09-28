//! ADFS SSO connection config, loaded from `services/web/config/adfs.yaml`.
//!
//! ADFS SSO is the only way a non-operator account comes into existence: the
//! domain allow-list and the AD group map are the two halves of the
//! provisioning gate. Operators are created out-of-band with
//! `admin users create`. `auto_provision` defaults off — an allow-listed domain
//! says an address *could* belong to someone, only a mapped AD group says IT
//! approved them — so SSO signs in existing accounts unless an installation
//! trusts its group map.

use std::collections::BTreeMap;

use serde::Deserialize;

// Why: AD FS default claim URIs for the standard issuance templates; an
// installation whose issuance rules name them differently overrides them.
pub(super) fn default_email_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".to_owned()
}

pub(super) fn default_name_attribute() -> String {
    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name".to_owned()
}

pub(super) fn default_groups_attribute() -> String {
    "http://schemas.xmlsoap.org/claims/Group".to_owned()
}

pub(super) fn default_allowed_domains() -> Vec<String> {
    vec![
        "astounddigital.com".to_owned(),
        "astoundcommerce.com".to_owned(),
    ]
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

// Why: AD FS relying-party trust registers these URLs character for character;
// the loader appends them to the profile's external URL so trust and requests
// agree.
pub const METADATA_PATH: &str = "/saml/metadata";
pub const ACS_PATH: &str = "/admin/auth/adfs/acs";

/// ADFS SSO connection config (SAML 2.0 relying party).
///
/// There is no client secret: trust runs on the `IdP` signing certificate in
/// the federation metadata at `idp_metadata_path` (relative to
/// `services/web/config/`), committed so rotation is a reviewable diff rather
/// than a fetch at sign-in. `entity_id` is our `<Audience>`/`<Issuer>` and
/// `acs_url` must equal the trust's registered ACS; both are derived from the
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
    pub idp_metadata_path: String,
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
            && !self.idp_metadata_xml.is_empty()
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

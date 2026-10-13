//! The pure half of a SAML sign-in: the subject and attribute statement of a
//! verified assertion, read into core's [`ExternalIdentity`] with the `IdP`'s
//! claim URIs left as they arrived.

use chrono::{DateTime, Utc};
use saml::{Identity, NameIdFormat};
use systemprompt::sso::ExternalIdentity;

use super::config::{EntraSamlConfig, GROUPS_OVERAGE_ATTRIBUTE};

#[derive(Debug, Clone, Default)]
pub struct AssertionClaims {
    pub name_id: Option<String>,
    pub name_id_is_email: bool,
    pub attributes: Vec<(String, Vec<String>)>,
}

impl AssertionClaims {
    #[must_use]
    pub fn from_identity(identity: &Identity) -> Self {
        Self {
            name_id: Some(identity.name_id.value.clone()),
            name_id_is_email: identity.name_id.format == NameIdFormat::EmailAddress,
            attributes: identity
                .attributes
                .iter()
                .map(|a| (a.name.clone(), a.values.clone()))
                .collect(),
        }
    }

    #[must_use]
    pub fn values(&self, attribute: &str) -> Vec<String> {
        self.attributes
            .iter()
            .filter(|(name, _)| name == attribute)
            .flat_map(|(_, values)| values.iter().cloned())
            .collect()
    }

    fn first(&self, attribute: &str) -> Option<String> {
        self.values(attribute)
            .into_iter()
            .map(|v| v.trim().to_owned())
            .find(|v| !v.is_empty())
    }

    // Why: Entra ID compares addresses case-insensitively and `users.email` is
    // stored lower-cased, so the asserted email is folded before any lookup.
    #[must_use]
    pub fn login_email(&self, cfg: &EntraSamlConfig) -> Option<String> {
        self.first(&cfg.email_attribute)
            .or_else(|| {
                self.name_id
                    .as_deref()
                    .filter(|_| self.name_id_is_email)
                    .map(ToOwned::to_owned)
            })
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains('@'))
    }

    #[must_use]
    pub fn display_name(&self, cfg: &EntraSamlConfig) -> Option<String> {
        self.first(&cfg.name_attribute)
    }

    // Why: SAML IdPs are often configured with claims but no usable NameID,
    // so the login email stands in as the stable external subject.
    #[must_use]
    pub fn external_sub(&self, email: &str) -> String {
        self.name_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(|| email.to_owned(), ToOwned::to_owned)
    }

    // Why: Microsoft Entra ID withholds the groups claim from a user in more
    // groups than a token carries and sends a Graph link in its place.
    #[must_use]
    pub fn groups_overage(&self) -> bool {
        self.attributes
            .iter()
            .any(|(name, _)| name == GROUPS_OVERAGE_ATTRIBUTE)
    }

    /// Entra ID is the corporate directory, so an address it asserts is
    /// verified by construction; the policy's domain list and group map gate.
    #[must_use]
    pub fn to_external_identity(
        &self,
        cfg: &EntraSamlConfig,
        assertion_id: Option<String>,
        assertion_expires_at: Option<DateTime<Utc>>,
    ) -> ExternalIdentity {
        let email = self.login_email(cfg);
        let raw_claims = self
            .attributes
            .iter()
            .map(|(name, values)| (name.clone(), serde_json::json!(values)))
            .collect::<serde_json::Map<_, _>>();
        ExternalIdentity {
            subject: self.external_sub(email.as_deref().unwrap_or_default()),
            email_verified: email.is_some(),
            email,
            display_name: self.display_name(cfg),
            groups: self.values(&cfg.groups_attribute),
            groups_overage: self.groups_overage(),
            tenant_id: None,
            assertion_id,
            assertion_expires_at,
            raw_claims: serde_json::Value::Object(raw_claims),
        }
    }
}

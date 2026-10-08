//! Salesforce connection config, loaded from
//! `services/web/config/salesforce.yaml`.
//!
//! This is no longer an SSO config — ADFS is the login. What remains is the
//! Connected App identity needed to mint a per-user bearer for the Salesforce
//! MCP server.

use serde::Deserialize;

pub(crate) fn salesforce_private_key() -> Option<String> {
    std::env::var("SALESFORCE_PRIVATE_KEY").ok().or_else(|| {
        systemprompt::config::SecretsBootstrap::get()
            .ok()
            .and_then(|s| s.get("salesforce_private_key").cloned())
    })
}

/// Salesforce Connected App config.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: Salesforce's JWT-bearer flow uses the org's My Domain base URL
    // (`https://astound.my.salesforce.com`) as the assertion audience.
    pub my_domain: String,
    #[serde(alias = "client_id")]
    pub consumer_key: String,
}

impl SalesforceConfig {
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            my_domain: String::new(),
            consumer_key: String::new(),
        }
    }

    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled && !self.my_domain.is_empty() && !self.consumer_key.is_empty()
    }

    fn base(&self) -> &str {
        self.my_domain.trim_end_matches('/')
    }

    pub(crate) fn token_url(&self) -> String {
        format!("{}/services/oauth2/token", self.base())
    }

    pub(crate) fn jwt_bearer_audience(&self) -> &str {
        self.base()
    }
}

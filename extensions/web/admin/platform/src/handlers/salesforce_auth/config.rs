//! The Salesforce app identity a JWT-bearer assertion is minted for, built
//! per org from `services/config/salesforce-orgs.yaml` at pre-authorization.

use serde::Deserialize;

/// Salesforce Connected App config.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: Salesforce's JWT-bearer flow uses the org's My Domain base URL
    // (`https://<my-domain>.my.salesforce.com`) as the assertion audience.
    pub my_domain: String,
    #[serde(alias = "client_id")]
    pub consumer_key: String,
}

impl SalesforceConfig {
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

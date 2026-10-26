//! Pinned provider resources and server-only application configuration.

use serde::{Deserialize, Serialize};
use systemprompt::manifest::ServicesConfig;
use systemprompt::models::auth::Permission;
use systemprompt::models::mcp::Deployment;
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

/// A configured MCP connector, named by its server id.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub enum Provider {
    Generic(String),
}

impl TryFrom<String> for Provider {
    type Error = String;
    fn try_from(id: String) -> Result<Self, Self::Error> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
        {
            return Err("Invalid connector identifier".into());
        }
        Ok(Self::Generic(id))
    }
}
impl From<Provider> for String {
    fn from(provider: Provider) -> Self {
        provider.slug().to_owned()
    }
}
impl Provider {
    pub fn slug(&self) -> &str {
        match self {
            Self::Generic(id) => id,
        }
    }
    pub fn endpoint(&self, services: &ServicesConfig) -> String {
        self.server(services)
            .and_then(|server| server.endpoint.clone())
            .unwrap_or_default()
    }
    pub fn callback(&self, config: &systemprompt::manifest::Config) -> String {
        format!(
            "{}/api/public/connectors/{}/callback",
            config.api_external_url.trim_end_matches('/'),
            self.slug(),
        )
    }
    pub fn configured(&self, services: &ServicesConfig) -> bool {
        self.server(services).is_some_and(|server| server.enabled)
    }
    pub fn requires_auth(&self, services: &ServicesConfig) -> bool {
        self.settings(services).is_some() || self.is_session_attested(services)
    }
    fn server<'a>(&self, services: &'a ServicesConfig) -> Option<&'a Deployment> {
        services.mcp_servers.get(self.slug())
    }
    pub fn settings(
        &self,
        services: &ServicesConfig,
    ) -> Option<systemprompt::models::mcp::deployment::ConnectorConfig> {
        self.server(services)?.connector.clone()
    }
    pub fn session_scopes<'a>(&self, services: &'a ServicesConfig) -> Option<&'a [Permission]> {
        if self.settings(services).is_some() {
            return None;
        }
        let oauth = &self.server(services)?.oauth;
        (oauth.required && !oauth.scopes.is_empty()).then_some(oauth.scopes.as_slice())
    }
    pub fn is_session_attested(&self, services: &ServicesConfig) -> bool {
        self.session_scopes(services).is_some()
    }
    pub fn display_name(&self, services: &ServicesConfig) -> String {
        self.settings(services)
            .and_then(|settings| settings.display_name)
            .unwrap_or_else(|| self.slug().to_owned())
    }
}

pub(crate) fn secret(secrets: &dyn SecretStore, name: &str) -> AdminResult<String> {
    secrets
        .secret(name)
        .map(|s| s.expose().to_owned())
        .filter(|s| !s.is_empty() && !s.starts_with("REPLACE_WITH"))
        .ok_or_else(|| AdminError::Unavailable(format!("Connector configuration missing: {name}")))
}

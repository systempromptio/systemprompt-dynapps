//! Pinned provider resources and server-only application configuration.

use serde::{Deserialize, Serialize};
use systemprompt::models::auth::Permission;
use systemprompt::models::mcp::Deployment;
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
    pub fn endpoint(&self) -> String {
        systemprompt::loader::ServicesBootstrap::get()
            .ok()
            .and_then(|s| s.mcp_servers.get(self.slug()))
            .and_then(|server| server.endpoint.clone())
            .unwrap_or_default()
    }
    pub fn callback(&self) -> AdminResult<String> {
        let cfg = systemprompt::manifest::Config::get().map_err(AdminError::internal)?;
        Ok(format!(
            "{}/api/public/connectors/{}/callback",
            cfg.api_external_url.trim_end_matches('/'),
            self.slug(),
        ))
    }
    pub fn configured(&self) -> bool {
        systemprompt::loader::ServicesBootstrap::get().is_ok_and(|services| {
            services
                .mcp_servers
                .get(self.slug())
                .is_some_and(|server| server.enabled)
        })
    }
    pub fn requires_auth(&self) -> bool {
        self.settings().is_some() || self.is_session_attested()
    }
    fn server(&self) -> Option<&'static Deployment> {
        systemprompt::loader::ServicesBootstrap::get()
            .ok()?
            .mcp_servers
            .get(self.slug())
    }
    pub fn settings(&self) -> Option<systemprompt::models::mcp::deployment::ConnectorConfig> {
        self.server()?.connector.clone()
    }
    pub fn session_scopes(&self) -> Option<&'static [Permission]> {
        if self.settings().is_some() {
            return None;
        }
        let oauth = &self.server()?.oauth;
        (oauth.required && !oauth.scopes.is_empty()).then_some(oauth.scopes.as_slice())
    }
    pub fn is_session_attested(&self) -> bool {
        self.session_scopes().is_some()
    }
    pub fn display_name(&self) -> String {
        self.settings()
            .and_then(|settings| settings.display_name)
            .unwrap_or_else(|| self.slug().to_owned())
    }
}

pub(crate) fn secret(name: &str) -> AdminResult<String> {
    systemprompt::config::SecretsBootstrap::get()
        .map_err(AdminError::internal)?
        .get(name)
        .filter(|s| !s.is_empty() && !s.starts_with("REPLACE_WITH"))
        .cloned()
        .ok_or_else(|| AdminError::Unavailable(format!("Connector configuration missing: {name}")))
}

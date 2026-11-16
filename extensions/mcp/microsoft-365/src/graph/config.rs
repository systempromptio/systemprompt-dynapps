//! Where Graph lives and how far a call may go: the endpoint (configurable
//! so a simulated Graph can stand in for Microsoft's) and the bounds on
//! retries, waits, pages and response size.
//!
//! The endpoint comes from the optional secret `microsoft_365_graph_url`
//! (default `https://graph.microsoft.com/v1.0`), read on every call so a
//! change applies without a restart, as `microsoft_365_authority` does. Tool
//! paths resolve under it and never carry a query; a URL Graph hands back
//! (`@odata.nextLink`) is admitted only on the same origin and path.

use std::time::Duration;

use systemprompt::client::{GuardedClientConfig, guarded_client};
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use systemprompt::traits::SecretLookup;
use url::Url;

use super::error::GraphError;

pub const DEFAULT_GRAPH_URL: &str = "https://graph.microsoft.com/v1.0";

pub const GRAPH_URL_SECRET: &str = "microsoft_365_graph_url";

const MAX_REDIRECTS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct GraphLimits {
    pub max_response_bytes: usize,
    pub max_throttle_retries: u32,
    pub max_retry_wait: Duration,
    pub default_backoff: Duration,
    pub max_pages: usize,
    pub timeout: Duration,
}

impl Default for GraphLimits {
    fn default() -> Self {
        Self {
            max_response_bytes: 25 * 1024 * 1024,
            max_throttle_retries: 2,
            max_retry_wait: Duration::from_secs(30),
            default_backoff: Duration::from_secs(2),
            max_pages: 20,
            timeout: Duration::from_secs(30),
        }
    }
}

// Why: core's guarded client checks every redirect hop against the SSRF
// guard and reqwest drops `Authorization` on a cross-host hop; Graph's
// `/content` answers 302 to a pre-signed URL that must not get the bearer.
pub fn http_client(limits: &GraphLimits) -> Result<reqwest::Client, GraphError> {
    let config = GuardedClientConfig::for_configured()
        .with_max_redirects(MAX_REDIRECTS)
        .with_timeout(limits.timeout);
    Ok(guarded_client(&config)?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEndpoint {
    base: Url,
}

impl GraphEndpoint {
    pub fn parse(raw: &str) -> Result<Self, GraphError> {
        let trimmed = raw.trim().trim_end_matches('/');
        let base = validate_outbound_url(trimmed, LoopbackPolicy::Allow)?;
        if base.query().is_some()
            || base.fragment().is_some()
            || !base.username().is_empty()
            || base.password().is_some()
        {
            return Err(GraphError::Config(
                "graph url must not carry a query, fragment or credentials".to_owned(),
            ));
        }
        Ok(Self { base })
    }

    pub fn from_lookup(secrets: &dyn SecretLookup) -> Result<Self, GraphError> {
        let configured = secrets
            .lookup(GRAPH_URL_SECRET)
            .map(|secret| secret.expose().trim().to_owned())
            .filter(|value| !value.is_empty());
        Self::parse(configured.as_deref().unwrap_or(DEFAULT_GRAPH_URL))
    }

    pub const fn base(&self) -> &Url {
        &self.base
    }

    fn base_path(&self) -> &str {
        self.base.path().trim_end_matches('/')
    }

    pub fn resolve(&self, path: &str) -> Result<Url, GraphError> {
        if !path.starts_with('/')
            || path.contains(['?', '#', '\\'])
            || path
                .split('/')
                .any(|segment| segment == ".." || segment == ".")
        {
            return Err(GraphError::Invalid(
                "graph path must be absolute, without query or dot segments".to_owned(),
            ));
        }
        let url = Url::parse(&format!(
            "{}{path}",
            self.base.as_str().trim_end_matches('/')
        ))?;
        self.admit(url)
    }

    pub fn admit_link(&self, link: &str) -> Result<Url, GraphError> {
        match Url::parse(link) {
            Ok(url) => self.admit(url),
            Err(error) => {
                tracing::warn!(%error, "Graph returned a nextLink that is not a URL");
                Err(GraphError::OffEndpoint)
            },
        }
    }

    fn admit(&self, url: Url) -> Result<Url, GraphError> {
        let base_path = self.base_path();
        let under_base = url.path() == base_path
            || url
                .path()
                .strip_prefix(base_path)
                .is_some_and(|rest| rest.starts_with('/'));
        if url.origin() != self.base.origin()
            || !under_base
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(GraphError::OffEndpoint);
        }
        Ok(url)
    }
}

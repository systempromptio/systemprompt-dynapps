//! The tenant's OpenID Connect discovery document, fetched once and kept
//! fresh in the background.
//!
//! The endpoints and the issuer come from the document rather than being
//! hand-built, so a tenant moved between Entra clouds needs no config change.
//! A cold cache (first sign-in before the background fetch landed) fetches
//! inline instead of refusing.

use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use systemprompt::sso::SsoError;

use super::super::refresh::RefreshingDoc;

const DISCOVERY_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const DISCOVERY_RETRY: Duration = Duration::from_secs(5 * 60);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Deserialize)]
pub struct OidcDiscovery {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
}

pub(super) fn http_client() -> Result<reqwest::Client, SsoError> {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(SsoError::internal)
}

async fn fetch_discovery(http: reqwest::Client, url: String) -> Result<OidcDiscovery, String> {
    let response = http
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("discovery request to {url} failed: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("discovery {url} answered {status}"));
    }
    response
        .json::<OidcDiscovery>()
        .await
        .map_err(|e| format!("discovery {url} unparseable: {e}"))
}

#[derive(Debug)]
pub(super) struct DiscoverySource {
    doc: RefreshingDoc<OidcDiscovery>,
}

impl DiscoverySource {
    pub(super) fn new(http: reqwest::Client, url: String) -> Self {
        let doc = RefreshingDoc::new(
            "entra-oidc discovery",
            move || fetch_discovery(http.clone(), url.clone()),
            DISCOVERY_TTL,
            DISCOVERY_RETRY,
        );
        Self { doc }
    }

    pub(super) fn spawn_refresh(&self) {
        self.doc.spawn_refresh();
    }

    pub(super) async fn current(&self) -> Result<Arc<OidcDiscovery>, SsoError> {
        if let Some(doc) = self.doc.get() {
            return Ok(doc);
        }
        self.doc.refresh().await.map_err(SsoError::Unavailable)
    }
}

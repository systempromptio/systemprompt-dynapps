//! The `IdP` federation metadata in force: the document fetched from
//! `idp_metadata_url` when one is configured, else the file the loader read
//! from `idp_metadata_path`.
//!
//! A fetch that fails keeps the last good document; with none, the pinned file
//! is used, and with no file either sign-in reports unavailable. Parsing is
//! cached per document, so it happens once per document rather than once per
//! sign-in.

use std::sync::Arc;
use std::time::Duration;

use saml::IdpDescriptor;
use systemprompt::sso::SsoError;
use tokio::sync::RwLock;

use super::config::EntraSamlConfig;
use crate::sso::refresh::RefreshingDoc;

// Why: Microsoft Entra ID publishes a new signing certificate in its
// federation metadata ahead of switching to it; six hours keeps well inside
// that notice without fetching on every sign-in.
pub const REFRESH_INTERVAL: Duration = Duration::from_hours(6);
const RETRY_AFTER_FAILURE: Duration = Duration::from_mins(5);
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) struct IdpMetadata {
    pinned: Arc<str>,
    remote: Option<RefreshingDoc<String>>,
    parsed: RwLock<Option<(Arc<str>, Arc<IdpDescriptor>)>>,
}

impl std::fmt::Debug for IdpMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdpMetadata")
            .field("remote", &self.remote)
            .finish_non_exhaustive()
    }
}

impl IdpMetadata {
    pub(super) fn new(cfg: &EntraSamlConfig) -> Self {
        let remote = cfg.idp_metadata_url.clone().map(|url| {
            RefreshingDoc::new(
                "entra-saml federation metadata",
                move || download(url.clone()),
                REFRESH_INTERVAL,
                RETRY_AFTER_FAILURE,
            )
        });
        Self {
            pinned: Arc::from(cfg.idp_metadata_xml.as_str()),
            remote,
            parsed: RwLock::new(None),
        }
    }

    pub(super) fn spawn_refresh(&self) {
        if let Some(remote) = &self.remote {
            remote.spawn_refresh();
        }
    }

    async fn current_xml(&self) -> Result<Arc<str>, SsoError> {
        if let Some(remote) = &self.remote {
            if let Some(xml) = remote.get() {
                return Ok(Arc::from(xml.as_str()));
            }
            if let Ok(xml) = remote.refresh().await {
                return Ok(Arc::from(xml.as_str()));
            }
        }
        if self.pinned.is_empty() {
            return Err(SsoError::Unavailable(
                "no idp_metadata_url document and no idp_metadata_path file".to_owned(),
            ));
        }
        Ok(Arc::clone(&self.pinned))
    }

    pub(super) async fn descriptor(&self) -> Result<Arc<IdpDescriptor>, SsoError> {
        let xml = self.current_xml().await?;
        if let Some((cached_xml, idp)) = self.parsed.read().await.as_ref()
            && *cached_xml == xml
        {
            return Ok(Arc::clone(idp));
        }
        let idp = Arc::new(
            IdpDescriptor::from_metadata_xml(xml.as_bytes())
                .map_err(|e| SsoError::Unavailable(format!("IdP metadata unusable: {e}")))?,
        );
        *self.parsed.write().await = Some((xml, Arc::clone(&idp)));
        Ok(idp)
    }
}

/// The `entityID` of a metadata document, for the issuer federated
/// identities are keyed by.
pub(super) fn entity_id_of(xml: &str) -> Option<String> {
    IdpDescriptor::from_metadata_xml(xml.as_bytes())
        .ok()
        .map(|idp| idp.entity_id)
}

async fn download(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;
    let xml = client
        .get(&url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| format!("IdP federation metadata fetch from {url} failed: {e}"))?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    IdpDescriptor::from_metadata_xml(xml.as_bytes())
        .map_err(|e| format!("IdP federation metadata from {url} does not parse: {e}"))?;
    Ok(xml)
}

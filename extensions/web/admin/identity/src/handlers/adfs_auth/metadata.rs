//! The `IdP` federation metadata in force: the document fetched from
//! `idp_metadata_url` when one is configured, else the file the loader read
//! from `idp_metadata_path`.
//!
//! A fetched document is held for [`REFRESH_INTERVAL`] and re-fetched after
//! it, so a signing certificate the `IdP` rolls over reaches sign-in without a
//! redeploy. A fetch that fails keeps the last good document; with none, the
//! pinned file is used, and with no file either sign-in reports unavailable.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use saml::IdpDescriptor;
use tokio::sync::RwLock;

use super::AdfsError;
use super::config::AdfsConfig;

// Why: Microsoft Entra ID publishes a new signing certificate in its
// federation metadata ahead of switching to it; six hours keeps well inside
// that notice without fetching on every sign-in.
pub const REFRESH_INTERVAL: Duration = Duration::from_hours(6);
const RETRY_AFTER_FAILURE: Duration = Duration::from_mins(5);
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

struct Fetched {
    xml: Arc<str>,
    next_fetch: Instant,
}

static FETCHED: LazyLock<RwLock<HashMap<String, Fetched>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub(super) async fn current_metadata_xml(cfg: &AdfsConfig) -> Result<Arc<str>, AdfsError> {
    let Some(url) = cfg.idp_metadata_url.as_deref() else {
        return pinned(cfg);
    };
    if let Some(entry) = FETCHED.read().await.get(url)
        && Instant::now() < entry.next_fetch
    {
        return Ok(Arc::clone(&entry.xml));
    }
    refresh(url).await.or_else(|e| {
        tracing::error!(error = %e, url, "IdP federation metadata fetch failed; using the pinned file");
        pinned(cfg)
    })
}

fn pinned(cfg: &AdfsConfig) -> Result<Arc<str>, AdfsError> {
    if cfg.idp_metadata_xml.is_empty() {
        return Err(AdfsError::Metadata(
            "no idp_metadata_url document and no idp_metadata_path file".to_owned(),
        ));
    }
    Ok(Arc::from(cfg.idp_metadata_xml.as_str()))
}

async fn refresh(url: &str) -> Result<Arc<str>, AdfsError> {
    match download(url).await {
        Ok(xml) => {
            let xml: Arc<str> = Arc::from(xml);
            FETCHED.write().await.insert(
                url.to_owned(),
                Fetched {
                    xml: Arc::clone(&xml),
                    next_fetch: Instant::now() + REFRESH_INTERVAL,
                },
            );
            tracing::info!(url, "Fetched IdP federation metadata");
            Ok(xml)
        },
        Err(e) => {
            let last_good = {
                let mut fetched = FETCHED.write().await;
                fetched.get_mut(url).map(|entry| {
                    entry.next_fetch = Instant::now() + RETRY_AFTER_FAILURE;
                    Arc::clone(&entry.xml)
                })
            };
            let Some(xml) = last_good else {
                return Err(e);
            };
            tracing::warn!(error = %e, url, "IdP federation metadata refresh failed; keeping the last good document");
            Ok(xml)
        },
    }
}

async fn download(url: &str) -> Result<String, AdfsError> {
    let client = reqwest::Client::builder().timeout(FETCH_TIMEOUT).build()?;
    let xml = client
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)?
        .text()
        .await?;
    IdpDescriptor::from_metadata_xml(xml.as_bytes()).map_err(AdfsError::MetadataParse)?;
    Ok(xml)
}

// Why: Microsoft Entra ID rolls its signing certificate on its own schedule;
// fetching at boot and on an interval keeps the first sign-in after either
// from waiting on the IdP. No-op when SSO is off, no URL is set, or no Tokio
// runtime is running.
pub fn spawn_idp_metadata_refresh(cfg: &AdfsConfig) {
    let Some(url) = cfg.idp_metadata_url.clone().filter(|_| cfg.enabled) else {
        return;
    };
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        tracing::warn!(
            url,
            "No runtime at boot; IdP metadata is fetched at first sign-in"
        );
        return;
    };
    runtime.spawn(async move {
        loop {
            let wait = match refresh(&url).await {
                Ok(_) => REFRESH_INTERVAL,
                Err(e) => {
                    tracing::error!(error = %e, url, "IdP federation metadata unavailable; sign-in falls back to the pinned file");
                    RETRY_AFTER_FAILURE
                },
            };
            tokio::time::sleep(wait).await;
        }
    });
}

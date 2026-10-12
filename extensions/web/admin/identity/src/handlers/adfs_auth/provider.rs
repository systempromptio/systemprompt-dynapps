//! The SAML roles built from [`AdfsConfig`]: our service-provider descriptor
//! and the `IdP` descriptor parsed from the federation metadata in force.
//!
//! Parsing is cached per metadata document, so it happens once per document
//! rather than once per sign-in; a refreshed document replaces the entry.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use saml::{
    DigestAlgorithm, IdpDescriptor, NameIdFormat, PeerCryptoPolicy, ServiceProvider,
    ServiceProviderConfig, SignatureAlgorithm, SpWantSigned, SsoResponseEndpoint,
};
use tokio::sync::RwLock;

use super::AdfsError;
use super::config::AdfsConfig;
use super::metadata::current_metadata_xml;

static IDP_CACHE: LazyLock<RwLock<HashMap<Arc<str>, Arc<IdpDescriptor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub(super) fn service_provider(cfg: &AdfsConfig) -> Result<ServiceProvider, AdfsError> {
    ServiceProvider::new(ServiceProviderConfig {
        entity_id: cfg.entity_id.clone(),
        acs: vec![SsoResponseEndpoint::post(cfg.acs_url.clone(), 0, true)],
        slo: vec![],
        name_id_formats: vec![NameIdFormat::EmailAddress, NameIdFormat::Persistent],
        signing_key: None,
        decryption_key: None,
        sign_authn_requests: false,
        // Why: SAML IdPs, Entra ID included, sign only the assertion by default,
        // not the outer Response; requiring a signed Response rejects logins.
        want_signed: SpWantSigned {
            response: false,
            assertions: true,
        },
        allow_unsolicited: cfg.allow_idp_initiated,
        default_peer_crypto_policy: PeerCryptoPolicy::strong_defaults(),
        outbound_signature_algorithm: SignatureAlgorithm::RsaSha256,
        outbound_digest_algorithm: DigestAlgorithm::Sha256,
    })
    .map_err(|e| AdfsError::ServiceProvider(e.to_string()))
}

pub(super) async fn idp_descriptor(cfg: &AdfsConfig) -> Result<Arc<IdpDescriptor>, AdfsError> {
    let xml = current_metadata_xml(cfg).await?;
    if let Some(idp) = IDP_CACHE.read().await.get(&xml) {
        return Ok(Arc::clone(idp));
    }
    let parsed = IdpDescriptor::from_metadata_xml(xml.as_bytes())
        .map_err(|e| AdfsError::Metadata(e.to_string()))?;
    let idp = Arc::new(parsed);
    {
        let mut cache = IDP_CACHE.write().await;
        cache.clear();
        cache.insert(xml, Arc::clone(&idp));
    }
    Ok(idp)
}

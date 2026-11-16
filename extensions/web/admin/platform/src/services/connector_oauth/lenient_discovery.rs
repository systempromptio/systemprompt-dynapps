//! The two discovery shapes a `lenient` connector accepts beyond RFC 9728 and
//! RFC 8414, as pure checks over already-trusted URLs.
//!
//! [`resource_matches`]: protected resource metadata names the configured
//! endpoint — the exact endpoint in either mode, or in `lenient` its bare
//! origin.
//!
//! [`advertised_metadata_document`]: in `lenient`, an `authorization_servers`
//! entry on the endpoint's origin whose path is a well-known metadata path,
//! optionally followed by an issuer path as RFC 8414 §3.1 inserts it, is that
//! metadata document; the issuer it must declare is the origin plus that
//! issuer path. `None` means the entry is read as an issuer, as in `strict`.
//!
//! Neither relaxes anything that follows: the issuer the metadata document
//! declares, PKCE S256 and the trusted origins of the authorization and token
//! endpoints are checked exactly as in `strict`.

use systemprompt_web_admin_core::repositories::connector_discovery::DiscoveryMode;
use url::Url;

const METADATA_DOCUMENT_PATHS: [&str; 2] = [
    "/.well-known/oauth-authorization-server",
    "/.well-known/openid-configuration",
];

/// An `authorization_servers` entry that names a metadata document on the
/// endpoint's origin, with the issuer that document must declare.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataDocument {
    pub issuer: String,
    pub url: Url,
}

#[must_use]
pub fn resource_matches(advertised: &str, endpoint: &Url, mode: DiscoveryMode) -> bool {
    let Ok(advertised) = Url::parse(advertised) else {
        return false;
    };
    advertised == *endpoint
        || (mode == DiscoveryMode::Lenient
            && advertised.path() == "/"
            && same_origin_without_extras(&advertised, endpoint))
}

#[must_use]
pub fn advertised_metadata_document(
    advertised: &Url,
    endpoint: &Url,
    mode: DiscoveryMode,
) -> Option<MetadataDocument> {
    if mode != DiscoveryMode::Lenient || !same_origin_without_extras(advertised, endpoint) {
        return None;
    }
    let issuer_path = METADATA_DOCUMENT_PATHS.iter().find_map(|document| {
        let rest = advertised.path().strip_prefix(document)?;
        (rest.is_empty() || (rest.starts_with('/') && rest.len() > 1)).then_some(rest)
    })?;
    let mut issuer = advertised.clone();
    issuer.set_path(issuer_path);
    Some(MetadataDocument {
        issuer: issuer.into(),
        url: advertised.clone(),
    })
}

fn same_origin_without_extras(candidate: &Url, endpoint: &Url) -> bool {
    candidate.origin() == endpoint.origin()
        && candidate.query().is_none()
        && candidate.fragment().is_none()
        && candidate.username().is_empty()
        && candidate.password().is_none()
}

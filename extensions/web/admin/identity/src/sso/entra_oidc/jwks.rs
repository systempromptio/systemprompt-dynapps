//! The tenant's signing key for one `kid`, read from its JWKS.
//!
//! Entra ID omits the optional `alg` member (RFC 7517 §4.4) and adds
//! `x5t`, `x5c`, `issuer` and `cloud_instance_name`; core's `Jwk` requires
//! `alg`, so its `JwksClient` refuses the whole document. Only `kid`, `n` and
//! `e` are read here — the algorithm is pinned to RS256 by the caller from the
//! token header, never taken from the key.

use serde::Deserialize;
use systemprompt::sso::SsoError;

use super::super::error::SsoProviderError;

#[derive(Debug, Deserialize)]
struct KeySet {
    keys: Vec<Key>,
}

#[derive(Debug, Deserialize)]
struct Key {
    #[serde(default)]
    kty: String,
    #[serde(default)]
    kid: Option<String>,
    #[serde(default)]
    n: Option<String>,
    #[serde(default)]
    e: Option<String>,
}

pub(super) struct RsaComponents {
    pub n: String,
    pub e: String,
}

pub(super) async fn fetch_rsa_key(
    http: &reqwest::Client,
    jwks_uri: &str,
    allowed_hosts: &[String],
    kid: &str,
) -> Result<RsaComponents, SsoError> {
    let url = reqwest::Url::parse(jwks_uri).map_err(|source| SsoProviderError::JwksUri {
        url: jwks_uri.to_owned(),
        source,
    })?;
    let host_allowed = scheme_is_acceptable(&url)
        && url
            .host_str()
            .is_some_and(|h| allowed_hosts.iter().any(|a| a.eq_ignore_ascii_case(h)));
    if !host_allowed {
        return Err(SsoError::Unavailable(format!(
            "JWKS uri {jwks_uri} is not on an allowed https host"
        )));
    }
    let set: KeySet = http
        .get(url)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|source| SsoProviderError::JwksFetch {
            url: jwks_uri.to_owned(),
            source,
        })?
        .json()
        .await
        .map_err(|source| SsoProviderError::JwksParse {
            url: jwks_uri.to_owned(),
            source,
        })?;
    set.keys
        .into_iter()
        .find(|k| k.kty == "RSA" && k.kid.as_deref() == Some(kid))
        .and_then(|k| Some(RsaComponents { n: k.n?, e: k.e? }))
        .ok_or_else(|| SsoError::InvalidAssertion("unknown kid".to_owned()))
}

// Why: matches core's `JwksClient` — plain http only to a loopback IdP, which
// is what the integration harness's fake Entra serves.
fn scheme_is_acceptable(url: &reqwest::Url) -> bool {
    match url.scheme() {
        "https" => true,
        "http" => match url.host() {
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            Some(url::Host::Domain(d)) => d.eq_ignore_ascii_case("localhost"),
            None => false,
        },
        _ => false,
    }
}

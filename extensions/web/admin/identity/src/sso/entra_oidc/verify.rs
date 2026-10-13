//! id_token validation: signature against the tenant's JWKS, the registered
//! claims, then the checks Entra ID leaves to the relying party.
//!
//! [`identity_from_claims`] is the pure half — nonce, tenant, subject, the
//! email and group rules — so it is testable without a network or a key.

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
// JSON: core's `ExternalIdentity::raw_claims` keeps the id_token's claims
// as issued, so they are decoded untyped before `IdTokenClaims` reads them.
use serde_json::Value;
use sha2::{Digest, Sha256};
use systemprompt::security::keys::{JwksClient, JwksClientError};
use systemprompt::sso::{ExternalIdentity, SsoError};

use super::super::error::SsoProviderError;
use super::claims::IdTokenClaims;
use super::config::EntraOidcConfig;
use super::discovery::OidcDiscovery;

pub(super) async fn validate_id_token(
    id_token: &str,
    cfg: &EntraOidcConfig,
    discovery: &OidcDiscovery,
    jwks: &JwksClient,
    expected_nonce: &str,
) -> Result<ExternalIdentity, SsoError> {
    let header = decode_header(id_token).map_err(SsoProviderError::IdTokenHeader)?;
    if header.alg != Algorithm::RS256 {
        return Err(SsoError::InvalidAssertion(
            "id_token must be RS256-signed".to_owned(),
        ));
    }
    let kid = header
        .kid
        .ok_or_else(|| SsoError::InvalidAssertion("id_token carries no kid".to_owned()))?;

    let jwk = jwks
        .fetch_at(&discovery.issuer, &discovery.jwks_uri, &kid)
        .await
        .map_err(|e| match e {
            JwksClientError::KeyNotFound { .. } => {
                SsoError::InvalidAssertion("unknown kid".to_owned())
            },
            other => SsoError::Unavailable(format!("JWKS resolution failed: {other}")),
        })?;
    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(SsoError::internal)?;

    let configured_issuer = cfg.issuer();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[discovery.issuer.as_str(), configured_issuer.as_str()]);
    validation.set_audience(&[cfg.entra_client_id.as_str()]);
    validation.set_required_spec_claims(&["exp", "iat", "iss", "aud"]);
    validation.leeway = cfg.clock_skew_seconds;

    let raw = decode::<Value>(id_token, &key, &validation)
        .map_err(SsoProviderError::IdTokenRejected)?
        .claims;
    let claims: IdTokenClaims =
        serde_json::from_value(raw.clone()).map_err(SsoProviderError::IdTokenClaims)?;

    let mut identity = identity_from_claims(&claims, cfg, expected_nonce, raw)?;
    if identity.assertion_id.is_none() {
        identity.assertion_id = Some(hex::encode(Sha256::digest(id_token.as_bytes())));
    }
    Ok(identity)
}

pub fn identity_from_claims(
    claims: &IdTokenClaims,
    cfg: &EntraOidcConfig,
    expected_nonce: &str,
    raw: Value,
) -> Result<ExternalIdentity, SsoError> {
    if expected_nonce.is_empty() || claims.nonce.as_deref() != Some(expected_nonce) {
        return Err(SsoError::InvalidAssertion("nonce".to_owned()));
    }
    if claims.tid.as_deref() != Some(cfg.entra_tenant_id.as_str()) {
        return Err(SsoError::InvalidAssertion("tenant".to_owned()));
    }
    if claims.sub.trim().is_empty() {
        return Err(SsoError::NoSubject);
    }

    let groups = raw
        .get(&cfg.groups_claim)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let groups_overage = claims
        .claim_names
        .as_ref()
        .is_some_and(|names| names.contains_key(&cfg.groups_claim));

    let asserted_email = claims.email.clone().filter(|email| !email.is_empty());
    let email_verified = asserted_email.is_some();
    let email = asserted_email.or_else(|| {
        claims
            .preferred_username
            .clone()
            .filter(|name| name.contains('@'))
    });

    Ok(ExternalIdentity {
        subject: claims.sub.clone(),
        email,
        email_verified,
        display_name: claims.name.clone(),
        groups,
        groups_overage,
        tenant_id: claims.tid.clone(),
        assertion_id: claims.jti.clone(),
        assertion_expires_at: DateTime::<Utc>::from_timestamp(claims.exp, 0),
        raw_claims: raw,
    })
}

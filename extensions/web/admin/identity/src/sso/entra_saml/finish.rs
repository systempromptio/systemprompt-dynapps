//! The assertion consumer half of [`EntraSamlProvider`]: verify the posted
//! `SAMLResponse` — XML-DSig against the metadata certificate, audience,
//! `Destination`, validity window, and `InResponseTo` against the request the
//! driver stored — and read it into an [`ExternalIdentity`].
//!
//! Replay is not checked here: the driver claims the assertion id until its
//! `NotOnOrAfter`.

use std::time::{Duration, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use saml::sp::LoginTracker;
use saml::{ConsumeResponse, ReplayMode, SsoResponseBinding, SsoResponseEndpoint};
use systemprompt::sso::{ExternalIdentity, SsoError, SsoReturn, SsoStateContext};

use super::claims::AssertionClaims;
use super::provider::EntraSamlProvider;

fn assertion_error(e: saml::Error) -> SsoError {
    match e {
        saml::Error::XmlParse(m) if m.contains("NameID") => SsoError::NoSubject,
        saml::Error::UnsolicitedNotAllowed => SsoError::UnsolicitedNotAllowed,
        other => SsoError::InvalidAssertion(other.to_string()),
    }
}

impl EntraSamlProvider {
    pub(super) async fn verify(
        &self,
        ret: &SsoReturn,
        state: Option<&SsoStateContext>,
    ) -> Result<ExternalIdentity, SsoError> {
        let cfg = &self.cfg;
        let encoded = ret
            .param("SAMLResponse")
            .ok_or_else(|| SsoError::InvalidAssertion("no SAMLResponse".to_owned()))?;
        let xml = STANDARD
            .decode(encoded.trim())
            .map_err(|e| SsoError::InvalidAssertion(format!("SAMLResponse is not base64: {e}")))?;
        let idp = self.metadata.descriptor().await?;
        let sp = self.service_provider()?;

        let tracker = state
            .and_then(|s| s.request_id.clone())
            .map(|request_id| LoginTracker {
                request_id,
                issued_at: SystemTime::now(),
                idp_entity_id: idp.entity_id.clone(),
                acs_endpoint: SsoResponseEndpoint::post(cfg.acs_url.clone(), 0, true),
                requested_authn_context: None,
                requested_name_id_format: None,
            });
        let relay_state = tracker.as_ref().and(ret.param("RelayState"));

        let identity = sp
            .consume_response(ConsumeResponse {
                idp: &idp,
                peer_crypto_policy: None,
                saml_response: &xml,
                binding: SsoResponseBinding::HttpPost,
                relay_state,
                tracker: tracker.as_ref(),
                expected_destination: &cfg.acs_url,
                now: SystemTime::now(),
                clock_skew: Duration::from_secs(cfg.clock_skew_seconds),
                replay_cache: None,
                replay_mode: ReplayMode::All,
                holder_of_key_cert: None,
            })
            .map_err(assertion_error)?;

        let expires_at: DateTime<Utc> = identity.not_on_or_after.into();
        Ok(
            AssertionClaims::from_identity(&identity).to_external_identity(
                cfg,
                Some(identity.assertion_id.clone()),
                Some(expires_at),
            ),
        )
    }
}

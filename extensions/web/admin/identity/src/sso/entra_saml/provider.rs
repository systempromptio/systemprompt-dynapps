//! [`EntraSamlProvider`]: SAML 2.0 Web Browser SSO against an Entra ID
//! enterprise application, as one of core's [`SsoProvider`]s.
//!
//! `start` builds the `AuthnRequest` (HTTP-Redirect binding) with a random
//! `RelayState`; core's driver stores both and binds them to the browser.
//! `finish` lives in [`super::finish`]; `metadata` is our SP descriptor.

use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use saml::{
    Binding, DigestAlgorithm, Dispatch, NameIdFormat, PeerCryptoPolicy, ServiceProvider,
    ServiceProviderConfig, SignatureAlgorithm, SpWantSigned, SsoResponseEndpoint, StartLogin,
};
use systemprompt::sso::{
    CookieSameSite, ExternalIdentity, SsoError, SsoIdentityPolicy, SsoMetadataDoc, SsoPaths,
    SsoProtocol, SsoProvider, SsoRedirect, SsoReturn, SsoStart, SsoStateContext, SsoStateSeed,
};

use super::ID;
use super::config::{ACS_PATH, EntraSamlConfig, METADATA_PATH, START_PATH};
use super::metadata::IdpMetadata;

#[derive(Debug)]
pub struct EntraSamlProvider {
    pub(super) cfg: Arc<EntraSamlConfig>,
    pub(super) issuer: String,
    pub(super) metadata: IdpMetadata,
}

impl EntraSamlProvider {
    #[must_use]
    pub fn new(cfg: Arc<EntraSamlConfig>, issuer: String) -> Self {
        let metadata = IdpMetadata::new(&cfg);
        metadata.spawn_refresh();
        Self {
            cfg,
            issuer,
            metadata,
        }
    }

    pub(super) fn service_provider(&self) -> Result<ServiceProvider, SsoError> {
        ServiceProvider::new(ServiceProviderConfig {
            entity_id: self.cfg.entity_id.clone(),
            acs: vec![SsoResponseEndpoint::post(self.cfg.acs_url.clone(), 0, true)],
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
            allow_unsolicited: self.cfg.allow_idp_initiated,
            default_peer_crypto_policy: PeerCryptoPolicy::strong_defaults(),
            outbound_signature_algorithm: SignatureAlgorithm::RsaSha256,
            outbound_digest_algorithm: DigestAlgorithm::Sha256,
        })
        .map_err(|e| SsoError::Unavailable(format!("service-provider configuration rejected: {e}")))
    }
}

fn random_relay_state() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

#[async_trait]
impl SsoProvider for EntraSamlProvider {
    fn id(&self) -> &str {
        ID
    }

    fn issuer(&self) -> &str {
        &self.issuer
    }

    fn protocol(&self) -> SsoProtocol {
        SsoProtocol::Saml2
    }

    fn policy(&self) -> &SsoIdentityPolicy {
        &self.cfg.policy
    }

    fn paths(&self) -> SsoPaths {
        SsoPaths {
            start: START_PATH,
            callback: ACS_PATH,
            metadata: Some(METADATA_PATH),
        }
    }

    fn login_path(&self) -> &str {
        "/admin/login"
    }

    fn default_return_to(&self) -> &str {
        "/admin"
    }

    fn allow_unsolicited(&self) -> bool {
        self.cfg.allow_idp_initiated
    }

    // Why: Microsoft Entra ID posts the ACS cross-site and browsers withhold
    // `SameSite=Lax` cookies on that POST, so the binder must be `None`.
    fn return_cookie_same_site(&self) -> CookieSameSite {
        CookieSameSite::None
    }

    async fn start(&self, _input: SsoStart<'_>) -> Result<SsoRedirect, SsoError> {
        let idp = self.metadata.descriptor().await?;
        let sp = self.service_provider()?;
        let relay_state = random_relay_state();
        let started = sp
            .start_login(
                &idp,
                StartLogin {
                    relay_state: Some(&relay_state),
                    binding: Binding::HttpRedirect,
                    force_authn: false,
                    is_passive: false,
                    requested_name_id_format: None,
                    requested_authn_context: None,
                    acs_index: None,
                    acs_url: None,
                    response_binding: None,
                },
            )
            .map_err(|e| SsoError::Unavailable(format!("AuthnRequest could not be built: {e}")))?;
        let Dispatch::Redirect(target) = started.dispatch else {
            return Err(SsoError::Unavailable(
                "SSO start produced a POST dispatch for a redirect binding".to_owned(),
            ));
        };
        Ok(SsoRedirect {
            url: target.as_str().to_owned(),
            seed: SsoStateSeed {
                state: relay_state,
                request_id: Some(started.tracker.request_id),
                nonce: None,
                pkce_verifier: None,
            },
        })
    }

    async fn finish(
        &self,
        ret: SsoReturn,
        state: Option<SsoStateContext>,
    ) -> Result<ExternalIdentity, SsoError> {
        let identity = self.verify(&ret, state.as_ref()).await?;
        Ok(identity)
    }

    async fn metadata(&self) -> Option<Result<SsoMetadataDoc, SsoError>> {
        Some(
            self.service_provider()
                .and_then(|sp| {
                    sp.metadata_xml(false)
                        .map_err(|e| SsoError::Unavailable(format!("SP metadata: {e}")))
                })
                .map(|body| SsoMetadataDoc {
                    content_type: "application/samlmetadata+xml",
                    body,
                }),
        )
    }
}

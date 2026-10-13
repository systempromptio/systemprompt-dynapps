//! A SAML `IdP` that signs real responses, for driving the ADFS / Entra SP.
//!
//! Responses are minted by the `saml` crate's own `IdP` role, so the bytes
//! the SP parses are the same shape a conforming `IdP` sends: an unsigned
//! `<samlp:Response>` around an RSA-SHA256, exclusive-C14N signed assertion,
//! which is what Entra ID emits and what the SP's `want_signed` asks for.
//!
//! Limitation: the crate's `IdP` role only answers an `AuthnRequest` -- its
//! `IssueResponse::in_response_to` is not optional and the lower-level
//! issuer is crate-private -- so an unsolicited (`IdP`-initiated) response
//! cannot be minted. [`SamlIdpFixture::issue_unsolicited_response`] returns
//! an error saying so; a flow test that needs one must skip explicitly.

use std::time::{Duration, SystemTime};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use saml::{
    AcsSelection, Attribute, AuthnContextClassRef, C14nAlgorithm, DataEncryptionAlgorithm,
    DigestAlgorithm, Endpoint, IdentityProvider, IdentityProviderConfig, IdpAssertionSigning,
    IssueResponse, KeyPair, KeyTransportAlgorithm, NameId, NameIdFormat, ParsedAuthnRequest,
    PeerCryptoPolicy, SignatureAlgorithm, SpDescriptor, SsoResponseDispatch, SsoResponseEndpoint,
    X509Certificate,
};

use super::keys::TestIdpKeys;

pub const EMAIL_ATTR: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress";
pub const NAME_ATTR: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name";
pub const GROUPS_ATTR: &str = "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups";
pub const GROUPS_LINK_ATTR: &str = "http://schemas.microsoft.com/claims/groups.link";

#[derive(Debug, Clone)]
pub struct SamlIdpFixture {
    pub entity_id: String,
    pub sso_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tamper {
    #[default]
    None,
    // Alter the signed assertion after signing, so the digest no longer
    // matches.
    BreakSignature,
    // Sign with [`TestIdpKeys::other`], a key the metadata does not carry.
    SignWithOtherKey,
}

#[derive(Debug, Clone)]
pub struct SamlResponseSpec {
    pub in_response_to: Option<String>,
    // The SP's ACS URL; becomes `Destination` and the bearer `Recipient`.
    pub destination: String,
    // The SP's entity id; becomes the `AudienceRestriction`.
    pub audience: String,
    pub name_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub groups_overage_link: bool,
    pub email_attr: String,
    pub name_attr: String,
    pub groups_attr: String,
    pub not_on_or_after: DateTime<Utc>,
    pub issue_instant: DateTime<Utc>,
    pub tamper: Tamper,
}

impl SamlResponseSpec {
    // A valid, five-minute response for `name_id` with Entra's claim names.
    pub fn new(
        destination: &str,
        audience: &str,
        in_response_to: Option<&str>,
        name_id: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            in_response_to: in_response_to.map(str::to_owned),
            destination: destination.to_owned(),
            audience: audience.to_owned(),
            name_id: name_id.to_owned(),
            email: Some(name_id.to_owned()),
            display_name: None,
            groups: Vec::new(),
            groups_overage_link: false,
            email_attr: EMAIL_ATTR.to_owned(),
            name_attr: NAME_ATTR.to_owned(),
            groups_attr: GROUPS_ATTR.to_owned(),
            not_on_or_after: now + chrono::Duration::minutes(5),
            issue_instant: now,
            tamper: Tamper::None,
        }
    }
}

impl Default for SamlIdpFixture {
    fn default() -> Self {
        Self {
            entity_id: "https://sts.fake-idp.test/tenant/".to_owned(),
            sso_url: "https://login.fake-idp.test/tenant/saml2".to_owned(),
        }
    }
}

impl SamlIdpFixture {
    // Federation metadata naming [`TestIdpKeys::get`] as the signing key.
    pub fn idp_metadata_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" xmlns:ds="http://www.w3.org/2000/09/xmldsig#" entityID="{entity}">
  <md:IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:KeyDescriptor use="signing">
      <ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo>
    </md:KeyDescriptor>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:2.0:nameid-format:persistent</md:NameIDFormat>
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="{sso}"/>
  </md:IDPSSODescriptor>
</md:EntityDescriptor>
"#,
            entity = self.entity_id,
            cert = TestIdpKeys::get().certificate_base64(),
            sso = self.sso_url,
        )
    }

    // The base64 `SAMLResponse` form value answering `spec.in_response_to`.
    //
    // # Panics
    // When `spec.in_response_to` is `None`; see the module head.
    pub fn issue_response(&self, spec: &SamlResponseSpec) -> String {
        let request_id = spec
            .in_response_to
            .as_deref()
            .expect("the saml crate cannot mint an unsolicited response; see module head");
        let keys = match spec.tamper {
            Tamper::SignWithOtherKey => TestIdpKeys::other(),
            Tamper::None | Tamper::BreakSignature => TestIdpKeys::get(),
        };
        let encoded = self.sign(spec, request_id, keys);
        match spec.tamper {
            Tamper::BreakSignature => break_signature(&encoded, &spec.name_id),
            Tamper::None | Tamper::SignWithOtherKey => encoded,
        }
    }

    // Always an error: the crate's `IdP` role cannot omit `InResponseTo`.
    pub fn issue_unsolicited_response(_spec: &SamlResponseSpec) -> Result<String, String> {
        Err("saml 0.0.1-alpha.2 IdP role cannot issue an unsolicited response".to_owned())
    }

    fn sign(&self, spec: &SamlResponseSpec, request_id: &str, keys: &TestIdpKeys) -> String {
        let idp = IdentityProvider::new(idp_config(&self.entity_id, &self.sso_url, keys))
            .expect("fake IdP config");
        let acs = SsoResponseEndpoint::post(spec.destination.clone(), 0, true);
        let sp = SpDescriptor {
            entity_id: spec.audience.clone(),
            assertion_consumer_services: vec![acs.clone()],
            single_logout_services: Vec::new(),
            signing_certs: Vec::new(),
            encryption_certs: Vec::new(),
            supported_name_id_formats: Vec::new(),
            want_assertions_signed: true,
            authn_requests_signed: false,
            valid_until: None,
            cache_duration: None,
        };
        let now = SystemTime::from(spec.issue_instant);
        let request = ParsedAuthnRequest {
            id: request_id.to_owned(),
            issuer: spec.audience.clone(),
            issue_instant: now,
            destination: Some(self.sso_url.clone()),
            assertion_consumer_service: acs,
            protocol_binding: None,
            assertion_consumer_service_selection: AcsSelection::Default,
            force_authn: false,
            is_passive: false,
            requested_name_id_format: None,
            requested_authn_context: None,
            relay_state: None,
        };
        let lifetime = (spec.not_on_or_after - spec.issue_instant)
            .to_std()
            .unwrap_or(Duration::from_secs(1));
        let dispatch = idp
            .issue_response(IssueResponse {
                sp: &sp,
                in_response_to: &request,
                name_id: NameId::new(spec.name_id.clone(), NameIdFormat::Persistent),
                attributes: attributes(spec),
                authn_instant: now,
                session_index: format!("_session_{request_id}"),
                session_not_on_or_after: None,
                authn_context_class_ref: AuthnContextClassRef::PasswordProtectedTransport,
                force_encrypt_assertion: Some(false),
                now,
                assertion_lifetime: lifetime,
                subject_confirmation_lifetime: lifetime,
                holder_of_key_cert: None,
            })
            .expect("issue SAML response");
        match dispatch {
            SsoResponseDispatch::Post(form) => form.saml_response,
            SsoResponseDispatch::Artifact(_) => panic!("ACS is HTTP-POST; got an artifact"),
        }
    }
}

fn idp_config(entity_id: &str, sso_url: &str, keys: &TestIdpKeys) -> IdentityProviderConfig {
    IdentityProviderConfig {
        entity_id: entity_id.to_owned(),
        sso: vec![Endpoint::redirect(sso_url, 0, true)],
        slo: Vec::new(),
        artifact_resolution: Vec::new(),
        supported_name_id_formats: vec![NameIdFormat::Persistent, NameIdFormat::EmailAddress],
        default_name_id_format: NameIdFormat::Persistent,
        signing_key: KeyPair::from_pkcs8_pem(keys.private_key_pkcs8_pem.as_bytes())
            .expect("IdP signing key")
            .with_certificate(
                X509Certificate::from_pem(keys.certificate_pem.as_bytes())
                    .expect("IdP certificate"),
            ),
        decryption_key: None,
        want_authn_requests_signed: false,
        assertion_signing: IdpAssertionSigning {
            sign_responses: false,
            sign_assertions: true,
        },
        encrypt_assertions_when_possible: false,
        default_session_duration: Duration::from_secs(3600),
        default_peer_crypto_policy: PeerCryptoPolicy::strong_defaults(),
        outbound_signature_algorithm: SignatureAlgorithm::RsaSha256,
        outbound_digest_algorithm: DigestAlgorithm::Sha256,
        outbound_c14n: C14nAlgorithm::ExclusiveCanonical,
        outbound_data_encryption_algorithm: DataEncryptionAlgorithm::Aes256Gcm,
        outbound_key_transport_algorithm: KeyTransportAlgorithm::RsaOaep,
    }
}

fn attributes(spec: &SamlResponseSpec) -> Vec<Attribute> {
    let mut out = Vec::new();
    if let Some(email) = &spec.email {
        out.push(Attribute::new(&spec.email_attr, vec![email.clone()]));
    }
    if let Some(name) = &spec.display_name {
        out.push(Attribute::new(&spec.name_attr, vec![name.clone()]));
    }
    if !spec.groups.is_empty() {
        out.push(Attribute::new(&spec.groups_attr, spec.groups.clone()));
    }
    if spec.groups_overage_link {
        let link = "https://graph.windows.net/tenant/users/subject/getMemberObjects";
        out.push(Attribute::new(GROUPS_LINK_ATTR, vec![link.to_owned()]));
    }
    out
}

fn break_signature(encoded: &str, name_id: &str) -> String {
    let xml = String::from_utf8(STANDARD.decode(encoded).expect("base64 response")).expect("utf8");
    let needle = format!(">{name_id}<");
    assert!(
        xml.contains(&needle),
        "NameID not found in the signed assertion"
    );
    STANDARD.encode(xml.replacen(&needle, &format!(">{name_id}.tampered<"), 1))
}

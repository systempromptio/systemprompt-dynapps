//! Process-wide RSA signing identities for the fake `IdP`s.
//!
//! RSA-2048 generation costs real time, so each identity is generated once
//! per test process and shared. [`TestIdpKeys::get`] is the trusted `IdP`;
//! [`TestIdpKeys::other`] is an unrelated key for wrong-signer cases.

use std::sync::OnceLock;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct TestIdpKeys {
    pub private_key_pkcs8_pem: String,
    pub public_key_pem: String,
    pub certificate_pem: String,
    pub kid: String,
    modulus: Vec<u8>,
    exponent: Vec<u8>,
}

static PRIMARY: OnceLock<TestIdpKeys> = OnceLock::new();
static OTHER: OnceLock<TestIdpKeys> = OnceLock::new();

impl TestIdpKeys {
    pub fn get() -> &'static Self {
        PRIMARY.get_or_init(|| Self::generate("fake-idp.test"))
    }

    pub fn other() -> &'static Self {
        OTHER.get_or_init(|| Self::generate("other-idp.test"))
    }

    fn generate(subject: &str) -> Self {
        let private = RsaPrivateKey::new(&mut rand_core::OsRng, 2048).expect("generate RSA key");
        let public = RsaPublicKey::from(&private);
        let private_key_pkcs8_pem = private
            .to_pkcs8_pem(LineEnding::LF)
            .expect("encode PKCS#8")
            .to_string();
        let public_key_pem = public
            .to_public_key_pem(LineEnding::LF)
            .expect("encode SPKI");
        let modulus = public.n().to_bytes_be();
        let exponent = public.e().to_bytes_be();
        let kid = URL_SAFE_NO_PAD.encode(&Sha256::digest(&modulus)[..12]);
        let certificate_pem = self_signed_certificate(&private_key_pkcs8_pem, subject);
        Self {
            private_key_pkcs8_pem,
            public_key_pem,
            certificate_pem,
            kid,
            modulus,
            exponent,
        }
    }

    // The certificate body as metadata carries it: base64 DER, no armour.
    pub fn certificate_base64(&self) -> String {
        self.certificate_pem
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect()
    }

    pub fn jwk(&self) -> serde_json::Value {
        serde_json::json!({
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": self.kid,
            "n": URL_SAFE_NO_PAD.encode(&self.modulus),
            "e": URL_SAFE_NO_PAD.encode(&self.exponent),
        })
    }

    pub fn jwks(&self) -> serde_json::Value {
        serde_json::json!({ "keys": [self.jwk()] })
    }

    pub fn encoding_key(&self) -> jsonwebtoken::EncodingKey {
        jsonwebtoken::EncodingKey::from_rsa_pem(self.private_key_pkcs8_pem.as_bytes())
            .expect("RSA encoding key")
    }
}

fn self_signed_certificate(private_key_pem: &str, subject: &str) -> String {
    let key =
        rcgen::KeyPair::from_pkcs8_pem_and_sign_algo(private_key_pem, &rcgen::PKCS_RSA_SHA256)
            .expect("import RSA key into rcgen");
    let params = rcgen::CertificateParams::new(vec![subject.to_owned()]).expect("cert params");
    params.self_signed(&key).expect("self-sign").pem()
}

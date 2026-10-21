//! How a minted ID token is framed: its header and its signature.
//!
//! The default is what Entra sends — RS256 under the tenant key's `kid` — and
//! every other form is a forgery a relying party must refuse.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::{Value, json};

use super::keys::TestIdpKeys;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TokenForm {
    #[default]
    Rs256,
    // `alg: none` with an empty signature segment.
    Unsigned,
    // HS256 keyed with the tenant's RSA public key PEM — the algorithm
    // confusion attack against verifiers that trust the header's `alg`.
    Hs256WithPublicKey,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum KidForm {
    #[default]
    SigningKey,
    Absent,
    Named(String),
}

#[derive(Debug, Clone, Default)]
pub struct HeaderSpec {
    pub form: TokenForm,
    pub kid: KidForm,
    pub crit: Option<Vec<String>>,
}

pub(super) fn sign(spec: &HeaderSpec, keys: &TestIdpKeys, claims: &Value) -> String {
    let kid = match &spec.kid {
        KidForm::SigningKey => Some(keys.kid.clone()),
        KidForm::Absent => None,
        KidForm::Named(kid) => Some(kid.clone()),
    };
    match spec.form {
        TokenForm::Unsigned => unsigned(kid, spec.crit.as_ref(), claims),
        TokenForm::Rs256 | TokenForm::Hs256WithPublicKey => {
            let hmac = spec.form == TokenForm::Hs256WithPublicKey;
            let mut header = Header::new(if hmac {
                Algorithm::HS256
            } else {
                Algorithm::RS256
            });
            header.kid = kid;
            header.crit.clone_from(&spec.crit);
            let key = if hmac {
                EncodingKey::from_secret(keys.public_key_pem.as_bytes())
            } else {
                keys.encoding_key()
            };
            jsonwebtoken::encode(&header, claims, &key).expect("sign ID token")
        },
    }
}

fn unsigned(kid: Option<String>, crit: Option<&Vec<String>>, claims: &Value) -> String {
    let mut header = json!({ "alg": "none", "typ": "JWT" });
    let obj = header.as_object_mut().expect("header object");
    if let Some(kid) = kid {
        obj.insert("kid".into(), json!(kid));
    }
    if let Some(crit) = crit {
        obj.insert("crit".into(), json!(crit));
    }
    let segment = |v: &Value| URL_SAFE_NO_PAD.encode(serde_json::to_vec(v).expect("json"));
    format!("{}.{}.", segment(&header), segment(claims))
}

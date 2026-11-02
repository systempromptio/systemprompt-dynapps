//! Bearer secrets the relay issues: device credentials, pairing tokens and
//! the secrets of the mobile sign-in.
//!
//! Each is a prefix plus 32 random bytes in unpadded base64url, and the relay
//! keeps only its SHA-256, so a database dump opens nothing. A device
//! credential (`dwr1_`) opens the device socket; a pairing token (`dwp1_`) is
//! redeemed once for a device credential. The mobile sign-in issues a mobile
//! credential (`dwm1_`), which opens the device socket and mints short-lived
//! JWTs, from a one-time code (`dwa1_`) that an approved consent request
//! (`dwq1_`) yields. None carries an identity, a name or an email, so none
//! leaks one through a QR, a log or a proxy.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

pub const CREDENTIAL_PREFIX: &str = "dwr1_";
pub const PAIRING_PREFIX: &str = "dwp1_";
pub const MOBILE_PREFIX: &str = "dwm1_";
pub const CODE_PREFIX: &str = "dwa1_";
pub const REQUEST_PREFIX: &str = "dwq1_";
const SECRET_LEN: usize = 32;
const ENCODED_LEN: usize = 43;

fn mint(prefix: &str) -> (String, CredentialHash) {
    let secret: [u8; SECRET_LEN] = rand::random();
    let token = format!("{prefix}{}", URL_SAFE_NO_PAD.encode(secret));
    let hash = CredentialHash::of(&token);
    (token, hash)
}

fn presented(prefix: &str, token: &str) -> Option<CredentialHash> {
    let encoded = token.strip_prefix(prefix)?;
    let shaped = encoded.len() == ENCODED_LEN
        && encoded
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    shaped.then(|| CredentialHash::of(token))
}

#[derive(Clone, PartialEq, Eq)]
pub struct DeviceCredential(String);

impl DeviceCredential {
    pub fn mint() -> (Self, CredentialHash) {
        let (token, hash) = mint(CREDENTIAL_PREFIX);
        (Self(token), hash)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DeviceCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DeviceCredential(<redacted>)")
    }
}

macro_rules! secret {
    ($name:ident, $prefix:expr) => {
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name(String);

        impl $name {
            pub fn mint() -> (Self, CredentialHash) {
                let (token, hash) = mint($prefix);
                (Self(token), hash)
            }

            pub fn expose(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "(<redacted>)"))
            }
        }
    };
}

secret!(PairingToken, PAIRING_PREFIX);
secret!(MobileCredential, MOBILE_PREFIX);
secret!(AuthorizationCode, CODE_PREFIX);
secret!(ConsentRequest, REQUEST_PREFIX);

/// The SHA-256 of a relay secret, as lowercase hex.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CredentialHash(String);

impl CredentialHash {
    fn of(token: &str) -> Self {
        Self(hex::encode(Sha256::digest(token.as_bytes())))
    }

    pub fn of_presented(token: &str) -> Option<Self> {
        presented(CREDENTIAL_PREFIX, token)
    }

    pub fn of_pairing_token(token: &str) -> Option<Self> {
        presented(PAIRING_PREFIX, token)
    }

    pub fn of_mobile_credential(token: &str) -> Option<Self> {
        presented(MOBILE_PREFIX, token)
    }

    pub fn of_code(token: &str) -> Option<Self> {
        presented(CODE_PREFIX, token)
    }

    pub fn of_consent_request(token: &str) -> Option<Self> {
        presented(REQUEST_PREFIX, token)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn from_stored(hex: String) -> Self {
        Self(hex)
    }
}

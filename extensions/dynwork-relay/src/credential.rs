//! Bearer secrets the relay issues: device credentials and pairing tokens.
//!
//! Both are a prefix plus 32 random bytes in unpadded base64url, and the
//! relay keeps only their SHA-256, so a database dump opens nothing. A device
//! credential (`dwr1_`) opens the device socket; a pairing token (`dwp1_`) is
//! redeemed once for a device credential. Neither carries an identity, a name
//! or an email, so neither leaks one through a QR, a log or a proxy.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

pub const CREDENTIAL_PREFIX: &str = "dwr1_";
pub const PAIRING_PREFIX: &str = "dwp1_";
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

#[derive(Clone, PartialEq, Eq)]
pub struct PairingToken(String);

impl PairingToken {
    pub fn mint() -> (Self, CredentialHash) {
        let (token, hash) = mint(PAIRING_PREFIX);
        (Self(token), hash)
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PairingToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairingToken(<redacted>)")
    }
}

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

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn from_stored(hex: String) -> Self {
        Self(hex)
    }
}

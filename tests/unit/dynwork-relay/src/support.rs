//! Fixtures shared by the unit suites.

use std::sync::{Arc, LazyLock};

use dynwork_relay::envelope::{HEADER_LEN, VERSION};
use dynwork_relay::ids::PeerId;
use systemprompt::models::auth::TokenAuthority;
use systemprompt_security::jwt::DeploymentTokenAuthority;
use systemprompt_security::keys::{RsaSigningKey, SigningKeys};

pub(crate) const ISSUER: &str = "https://relay.test";

static AUTHORITY: LazyLock<Arc<DeploymentTokenAuthority>> = LazyLock::new(|| {
    let key = RsaSigningKey::generate().expect("generate an RSA key");
    let keys = SigningKeys::new(key, &[7; 32]).expect("build signing keys");
    Arc::new(DeploymentTokenAuthority::new(keys, ISSUER))
});

pub(crate) fn authority() -> Arc<dyn TokenAuthority> {
    let authority: Arc<DeploymentTokenAuthority> = Arc::clone(&AUTHORITY);
    authority
}

pub(crate) fn peer(byte: u8) -> PeerId {
    PeerId::from_bytes([byte; 16])
}

pub(crate) fn envelope(to: PeerId, from: PeerId, ciphertext: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    bytes.push(VERSION);
    bytes.extend_from_slice(to.as_bytes());
    bytes.extend_from_slice(from.as_bytes());
    bytes.extend_from_slice(&7u32.to_be_bytes());
    bytes.extend_from_slice(ciphertext);
    bytes
}

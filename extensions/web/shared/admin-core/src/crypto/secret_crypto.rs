//! ChaCha20-Poly1305 primitives and master key loading for secret storage.
//!
//! The master key is the `encryption_master_key` secret and nothing else:
//! core's secrets store is the one source, and the key is decoded by the
//! same `decode_master_key` core uses for its own at-rest sealing, so an
//! operator sees one error shape whichever side refuses the material.

use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use systemprompt::config::{SecretsBootstrapError, decode_master_key};
use systemprompt::traits::SecretStore;

const MASTER_KEY_SECRET: &str = "encryption_master_key";

#[derive(Debug, thiserror::Error)]
pub enum SecretCryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(#[source] chacha20poly1305::Error),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(#[source] chacha20poly1305::Error),
    #[error("Master key not configured: add `encryption_master_key` to the secrets file")]
    MasterKeyMissing,
    #[error("Invalid key material: {0}")]
    InvalidKeyMaterial(#[source] SecretsBootstrapError),
    #[error("Stored key material has the wrong length: expected {expected} bytes, found {actual}")]
    StoredKeyLength { expected: usize, actual: usize },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[must_use]
pub fn generate_dek() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

#[must_use]
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(SecretCryptoError::EncryptionFailed)
}

pub fn decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
) -> Result<Vec<u8>, SecretCryptoError> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(SecretCryptoError::DecryptionFailed)
}

pub fn load_master_key(secrets: &dyn SecretStore) -> Result<[u8; 32], SecretCryptoError> {
    let secret = secrets.secret(MASTER_KEY_SECRET);
    master_key_from_secret(
        secret
            .as_ref()
            .map(systemprompt::traits::ExtensionSecret::expose),
    )
}

pub fn master_key_from_secret(encoded: Option<&str>) -> Result<[u8; 32], SecretCryptoError> {
    let encoded = encoded
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(SecretCryptoError::MasterKeyMissing)?;
    decode_master_key(encoded).map_err(SecretCryptoError::InvalidKeyMaterial)
}

pub fn fixed_key_material<const N: usize>(bytes: Vec<u8>) -> Result<[u8; N], SecretCryptoError> {
    <[u8; N]>::try_from(bytes).map_err(|bytes| SecretCryptoError::StoredKeyLength {
        expected: N,
        actual: bytes.len(),
    })
}

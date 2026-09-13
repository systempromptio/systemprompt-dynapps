//! Dedicated workflow signing keys and portable receipt verification.
use crate::error::{RequirementsError, Result};
use crate::model::Receipt;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub struct ReceiptSigner(SigningKey);

impl std::fmt::Debug for ReceiptSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceiptSigner").finish_non_exhaustive()
    }
}

impl ReceiptSigner {
    pub fn from_seed_base64(seed: &str) -> Result<Self> {
        let bytes = STANDARD.decode(seed)?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|v: Vec<u8>| RequirementsError::InvalidSeed(v.len()))?;
        Ok(Self(SigningKey::from_bytes(&seed)))
    }

    pub fn public_key(&self) -> String {
        STANDARD.encode(self.0.verifying_key().as_bytes())
    }
    pub fn key_id(&self) -> String {
        hex::encode(Sha256::digest(self.0.verifying_key().as_bytes()))
    }

    pub fn sign<T: Serialize>(&self, payload: &T) -> Result<Receipt> {
        let payload = serde_jcs::to_string(payload)?;
        let signature = STANDARD.encode(self.0.sign(payload.as_bytes()).to_bytes());
        Ok(Receipt {
            payload,
            signature,
            key_id: self.key_id(),
        })
    }
}

pub fn verify(receipt: &Receipt, public_key: &str) -> Result<()> {
    let bytes: [u8; 32] = STANDARD
        .decode(public_key)?
        .try_into()
        .map_err(|v: Vec<u8>| RequirementsError::InvalidPublicKey(v.len()))?;
    if hex::encode(Sha256::digest(bytes)) != receipt.key_id {
        return Err(RequirementsError::WrongReceiptKey);
    }
    let signature = Signature::from_slice(&STANDARD.decode(&receipt.signature)?)?;
    VerifyingKey::from_bytes(&bytes)?.verify(receipt.payload.as_bytes(), &signature)?;
    Ok(())
}

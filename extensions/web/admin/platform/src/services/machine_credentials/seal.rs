//! The envelope every stored credential secret is sealed in.
//!
//! The secret is encrypted with the master key together with the user and
//! provider it belongs to, and opening checks both, so a row copied onto
//! another account or provider cannot open.

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::crypto::secret_crypto;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Serialize, Deserialize)]
struct SealedSecret {
    user: String,
    provider: String,
    secret: String,
}

impl std::fmt::Debug for SealedSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SealedSecret { <redacted> }")
    }
}

#[derive(Debug)]
pub(super) struct Sealed {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub(super) fn seal(
    secrets: &dyn SecretStore,
    user: &UserId,
    provider: &str,
    secret: String,
) -> AdminResult<Sealed> {
    let key = secret_crypto::load_master_key(secrets)?;
    let nonce = secret_crypto::generate_nonce();
    let envelope = serde_json::to_vec(&SealedSecret {
        user: user.as_str().to_owned(),
        provider: provider.to_owned(),
        secret,
    })
    .map_err(AdminError::internal)?;
    let ciphertext = secret_crypto::encrypt(&key, &nonce, &envelope)?;
    Ok(Sealed {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

pub(super) fn unseal(
    secrets: &dyn SecretStore,
    ciphertext: &[u8],
    nonce: &[u8],
    user: &UserId,
    provider: &str,
) -> AdminResult<String> {
    let key = secret_crypto::load_master_key(secrets)?;
    let nonce = nonce.try_into().map_err(AdminError::internal)?;
    let bytes = secret_crypto::decrypt(&key, &nonce, ciphertext)?;
    let sealed: SealedSecret = serde_json::from_slice(&bytes).map_err(AdminError::internal)?;
    if sealed.user != user.as_str() || sealed.provider != provider {
        return Err(AdminError::Forbidden("Credential owner mismatch".into()));
    }
    Ok(sealed.secret)
}

//! Phone credentials, behind a verifier chain.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::AuthError;
use crate::credential::CredentialHash;
use crate::ids::{DeviceId, PeerId};
use crate::store::RelayStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedDevice {
    pub user: UserId,
    pub device: DeviceId,
    pub peer: PeerId,
    pub expires_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait DeviceVerifier: Send + Sync + Debug {
    async fn verify(&self, token: &str) -> Result<Option<VerifiedDevice>, AuthError>;
}

pub async fn verify_device(
    verifiers: &[Arc<dyn DeviceVerifier>],
    token: &str,
) -> Result<VerifiedDevice, AuthError> {
    for verifier in verifiers {
        if let Some(device) = verifier.verify(token).await? {
            return Ok(device);
        }
    }
    Err(AuthError::UnknownDevice)
}

/// The `dwr1_` credentials a pairing claim issues, looked up by hash.
#[derive(Debug, Clone)]
pub struct CredentialVerifier {
    store: Arc<dyn RelayStore>,
}

impl CredentialVerifier {
    pub fn new(store: Arc<dyn RelayStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl DeviceVerifier for CredentialVerifier {
    async fn verify(&self, token: &str) -> Result<Option<VerifiedDevice>, AuthError> {
        let Some(hash) = CredentialHash::of_presented(token) else {
            return Ok(None);
        };
        let device = self
            .store
            .find_phone_by_credential(&hash)
            .await?
            .ok_or(AuthError::UnknownDevice)?;
        Ok(Some(VerifiedDevice {
            user: device.user,
            device: device.id,
            peer: device.peer,
            expires_at: None,
        }))
    }
}

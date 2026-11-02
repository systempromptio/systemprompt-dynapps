//! The device verifier for mobile credentials (`dwm1_`).
//!
//! It is the second link in the chain [`crate::auth::verify_device`] runs, so
//! a phone signed in through the instance opens the device socket like a
//! paired one.

use std::sync::Arc;

use async_trait::async_trait;

use crate::auth::{AuthError, DeviceVerifier, VerifiedDevice};
use crate::credential::CredentialHash;
use crate::store::RelayStore;

#[derive(Debug, Clone)]
pub struct MobileCredentialVerifier {
    store: Arc<dyn RelayStore>,
}

impl MobileCredentialVerifier {
    pub fn new(store: Arc<dyn RelayStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl DeviceVerifier for MobileCredentialVerifier {
    async fn verify(&self, token: &str) -> Result<Option<VerifiedDevice>, AuthError> {
        let Some(hash) = CredentialHash::of_mobile_credential(token) else {
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

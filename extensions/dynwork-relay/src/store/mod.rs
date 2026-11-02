//! Persistence behind one trait: devices, pairings and, through its
//! [`MobileAuthStore`] supertrait, the mobile sign-in.
//!
//! The server runs over [`postgres::PgRelayStore`]; the trait is the seam the
//! test suites use to run the relay over an in-memory store. Every time the
//! store compares against "now" it takes it as an argument, so expiry follows
//! the injected clock.

pub mod mobile_auth;
mod mobile_auth_pg;
pub mod postgres;
mod rows;
mod types;

use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::credential::CredentialHash;
use crate::ids::{DeviceId, PairingId, PeerId, PeerKind};

pub use mobile_auth::{
    Account, CodeRedemption, DecisionOutcome, GrantDecision, GrantOutcome, MobileAuthStore,
    NewGrant, RedeemOutcome,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("relay store query failed")]
    Database(#[from] sqlx::Error),
    #[error("relay store row is invalid: {0}")]
    Corrupt(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceRecord {
    pub id: DeviceId,
    pub user: UserId,
    pub kind: PeerKind,
    pub peer: PeerId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DesktopRegistration {
    pub user: UserId,
    pub peer: PeerId,
    pub now: DateTime<Utc>,
    pub max_devices: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterOutcome {
    Registered(DeviceRecord),
    DeviceLimit,
    PeerTaken,
    UnknownUser,
}

#[derive(Debug, Clone)]
pub struct NewPairing {
    pub id: PairingId,
    pub user: UserId,
    pub token: CredentialHash,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PhoneClaim {
    pub token: CredentialHash,
    pub device: DeviceId,
    pub peer: PeerId,
    pub name: String,
    pub credential: CredentialHash,
    pub now: DateTime<Utc>,
    pub max_devices: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimOutcome {
    Claimed {
        device: DeviceRecord,
        pairing: PairingId,
    },
    Unknown,
    Used,
    Expired,
    DeviceLimit,
    PeerTaken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingOutcome {
    Inserted,
    TooMany,
    UnknownUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeOutcome {
    Revoked,
    Desktop,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevokedDevice {
    pub device: DeviceId,
    pub user: UserId,
    pub revoked_at: DateTime<Utc>,
}

/// The relay's storage: devices, pairings and revocations. `#[async_trait]`
/// because the relay state holds it as an `Arc<dyn RelayStore>` (Postgres in
/// production, memory in tests).
#[async_trait]
pub trait RelayStore: MobileAuthStore + Send + Sync + Debug {
    async fn register_desktop(
        &self,
        registration: &DesktopRegistration,
    ) -> Result<RegisterOutcome, StoreError>;

    async fn count_open_pairings(
        &self,
        user: &UserId,
        now: DateTime<Utc>,
    ) -> Result<i64, StoreError>;

    async fn insert_pairing(
        &self,
        pairing: &NewPairing,
        max_open: u32,
    ) -> Result<PairingOutcome, StoreError>;

    async fn claim_pairing(&self, claim: &PhoneClaim) -> Result<ClaimOutcome, StoreError>;

    async fn find_phone_by_credential(
        &self,
        credential: &CredentialHash,
    ) -> Result<Option<DeviceRecord>, StoreError>;

    async fn list_devices(&self, user: &UserId) -> Result<Vec<DeviceRecord>, StoreError>;

    async fn revoke_device(
        &self,
        user: &UserId,
        device: DeviceId,
        now: DateTime<Utc>,
    ) -> Result<RevokeOutcome, StoreError>;

    async fn list_revoked_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<RevokedDevice>, StoreError>;

    async fn touch_device(&self, device: DeviceId, now: DateTime<Utc>) -> Result<(), StoreError>;

    async fn delete_expired_pairings(&self, before: DateTime<Utc>) -> Result<u64, StoreError>;
}

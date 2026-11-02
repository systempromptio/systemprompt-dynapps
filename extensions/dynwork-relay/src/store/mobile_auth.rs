//! Persistence of the mobile sign-in: consent grants, their one-time codes,
//! and the accounts a mobile JWT is minted for.
//!
//! A grant moves through three steps, each a single locked transaction:
//! insert (the consent page is shown), decide (Allow or Deny), redeem (the
//! phone trades the code and its PKCE verifier for a device). Redeeming
//! creates the phone's device row in the same transaction, so a code yields
//! at most one device. A code presented again revokes the device its first
//! use created, since only a leaked code is presented twice.
//!
//! A redemption carries the S256 of the verifier the phone presented, to
//! compare with the challenge the grant was opened with; a sign-in reports the
//! phone's previous live device for the same peer, which it revoked.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::{DeviceRecord, StoreError};
use crate::credential::CredentialHash;
use crate::ids::{DeviceId, GrantId, PeerId};

/// A signed-in, active person: who a mobile JWT is minted for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub user: UserId,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NewGrant {
    pub id: GrantId,
    pub user: UserId,
    pub request: CredentialHash,
    pub challenge: String,
    pub state: String,
    pub device_name: String,
    pub now: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_open: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantOutcome {
    Inserted,
    TooMany,
    UnknownUser,
}

/// Allow carries the hash of the code it issues and that code's expiry;
/// Deny carries neither.
#[derive(Debug, Clone)]
pub struct GrantDecision {
    pub request: CredentialHash,
    pub user: UserId,
    pub code: Option<(CredentialHash, DateTime<Utc>)>,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionOutcome {
    Approved { state: String },
    Denied { state: String },
    Unknown,
    Expired,
    Decided,
}

#[derive(Debug, Clone)]
pub struct CodeRedemption {
    pub code: CredentialHash,
    pub challenge: String,
    pub device: DeviceId,
    pub peer: PeerId,
    pub credential: CredentialHash,
    pub now: DateTime<Utc>,
    pub max_devices: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedeemOutcome {
    SignedIn {
        device: DeviceRecord,
        account: Account,
        replaced: Option<DeviceId>,
    },
    Unknown,
    Used {
        user: UserId,
        revoked: Option<DeviceId>,
    },
    Expired,
    Mismatch,
    UnknownUser,
    DeviceLimit,
    PeerTaken,
}

#[async_trait]
pub trait MobileAuthStore: Send + Sync {
    async fn insert_grant(&self, grant: &NewGrant) -> Result<GrantOutcome, StoreError>;

    async fn decide_grant(&self, decision: &GrantDecision) -> Result<DecisionOutcome, StoreError>;

    async fn redeem_code(&self, redemption: &CodeRedemption) -> Result<RedeemOutcome, StoreError>;

    async fn find_account(&self, user: &UserId) -> Result<Option<Account>, StoreError>;

    async fn delete_expired_grants(&self, before: DateTime<Utc>) -> Result<u64, StoreError>;
}

//! The in-memory store's mobile sign-in, with `PgRelayStore`'s rules:
//! `postgres_mobile.rs` holds the real store to the same outcomes.

#![expect(
    clippy::significant_drop_tightening,
    reason = "each method is one critical section over the store, as a transaction is in Postgres"
)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dynwork_relay::credential::CredentialHash;
use dynwork_relay::ids::{DeviceId, GrantId, PeerKind};
use dynwork_relay::store::{
    Account, CodeRedemption, DecisionOutcome, DeviceRecord, GrantDecision, GrantOutcome,
    MobileAuthStore, NewGrant, RedeemOutcome, StoreError,
};
use systemprompt::identifiers::UserId;

use crate::memory_store::{Mem, MemoryStore};

#[derive(Debug, Clone)]
pub(crate) struct Grant {
    id: GrantId,
    user: UserId,
    request: CredentialHash,
    code: Option<CredentialHash>,
    challenge: String,
    state: String,
    device_name: String,
    expires_at: DateTime<Utc>,
    decided: bool,
    redeemed: bool,
    device: Option<DeviceId>,
}

fn account(mem: &Mem, user: &UserId) -> Option<Account> {
    (mem.users.contains(user) && !mem.inactive.contains(user)).then(|| Account {
        user: user.clone(),
        name: user.to_string(),
        email: format!("{user}@example.test"),
        roles: vec!["user".to_owned()],
    })
}

fn revoke(mem: &mut Mem, device: DeviceId, now: DateTime<Utc>) -> bool {
    mem.devices
        .iter_mut()
        .find(|(record, _)| record.id == device && record.revoked_at.is_none())
        .map(|(record, _)| record.revoked_at = Some(now))
        .is_some()
}

impl MemoryStore {
    pub(crate) fn deactivate(&self, user: &UserId) {
        self.mem.lock().unwrap().inactive.insert(user.clone());
    }
}

fn sign_in(mem: &mut Mem, grant: &Grant, redemption: &CodeRedemption) -> RedeemOutcome {
    let Some(account) = account(mem, &grant.user) else {
        return RedeemOutcome::UnknownUser;
    };
    let existing = mem
        .live(&grant.user)
        .find(|device| device.peer == redemption.peer)
        .cloned();
    let replaced = match existing {
        Some(device) if device.kind == PeerKind::Desktop => return RedeemOutcome::PeerTaken,
        other => other.map(|device| device.id),
    };
    let live = mem.live(&grant.user).count() - usize::from(replaced.is_some());
    if live >= redemption.max_devices as usize {
        return RedeemOutcome::DeviceLimit;
    }
    if let Some(previous) = replaced {
        revoke(mem, previous, redemption.now);
    }
    let device = DeviceRecord {
        id: redemption.device,
        user: grant.user.clone(),
        kind: PeerKind::Phone,
        peer: redemption.peer,
        name: grant.device_name.clone(),
        created_at: redemption.now,
        last_seen_at: Some(redemption.now),
        revoked_at: None,
    };
    mem.devices
        .push((device.clone(), Some(redemption.credential.clone())));
    if let Some(row) = mem.grants.iter_mut().find(|row| row.id == grant.id) {
        row.device = Some(redemption.device);
    }
    RedeemOutcome::SignedIn {
        device,
        account,
        replaced,
    }
}

#[async_trait]
impl MobileAuthStore for MemoryStore {
    async fn insert_grant(&self, grant: &NewGrant) -> Result<GrantOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        if !mem.users.contains(&grant.user) {
            return Ok(GrantOutcome::UnknownUser);
        }
        let open = mem
            .grants
            .iter()
            .filter(|row| row.user == grant.user && !row.decided && row.expires_at > grant.now)
            .count();
        if open >= grant.max_open as usize {
            return Ok(GrantOutcome::TooMany);
        }
        mem.grants.push(Grant {
            id: grant.id,
            user: grant.user.clone(),
            request: grant.request.clone(),
            code: None,
            challenge: grant.challenge.clone(),
            state: grant.state.clone(),
            device_name: grant.device_name.clone(),
            expires_at: grant.expires_at,
            decided: false,
            redeemed: false,
            device: None,
        });
        Ok(GrantOutcome::Inserted)
    }

    async fn decide_grant(&self, decision: &GrantDecision) -> Result<DecisionOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        let Some(grant) = mem
            .grants
            .iter_mut()
            .find(|row| row.request == decision.request && row.user == decision.user)
        else {
            return Ok(DecisionOutcome::Unknown);
        };
        if grant.decided {
            return Ok(DecisionOutcome::Decided);
        }
        if grant.expires_at <= decision.now {
            return Ok(DecisionOutcome::Expired);
        }
        grant.decided = true;
        Ok(match &decision.code {
            Some((code, expires_at)) => {
                grant.code = Some(code.clone());
                grant.expires_at = *expires_at;
                DecisionOutcome::Approved {
                    state: grant.state.clone(),
                }
            },
            None => DecisionOutcome::Denied {
                state: grant.state.clone(),
            },
        })
    }

    async fn redeem_code(&self, redemption: &CodeRedemption) -> Result<RedeemOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        let Some(grant) = mem
            .grants
            .iter()
            .find(|row| row.code.as_ref() == Some(&redemption.code))
            .cloned()
        else {
            return Ok(RedeemOutcome::Unknown);
        };
        if grant.redeemed {
            let revoked = grant
                .device
                .filter(|device| revoke(&mut mem, *device, redemption.now));
            return Ok(RedeemOutcome::Used {
                user: grant.user,
                revoked,
            });
        }
        if let Some(row) = mem.grants.iter_mut().find(|row| row.id == grant.id) {
            row.redeemed = true;
        }
        Ok(if grant.expires_at <= redemption.now {
            RedeemOutcome::Expired
        } else if grant.challenge != redemption.challenge {
            RedeemOutcome::Mismatch
        } else {
            sign_in(&mut mem, &grant, redemption)
        })
    }

    async fn find_account(&self, user: &UserId) -> Result<Option<Account>, StoreError> {
        Ok(account(&self.mem.lock().unwrap(), user))
    }

    async fn delete_expired_grants(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        let start = mem.grants.len();
        mem.grants.retain(|row| row.expires_at >= before);
        Ok((start - mem.grants.len()) as u64)
    }
}

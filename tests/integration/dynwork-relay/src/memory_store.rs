//! An in-memory `RelayStore` with the Postgres store's rules, so the socket
//! and route suites run without a database. `postgres_store.rs` holds the
//! real store to the same rules.

#![expect(
    clippy::significant_drop_tightening,
    reason = "each method is one critical section over the store, as a transaction is in Postgres"
)]

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dynwork_relay::credential::CredentialHash;
use dynwork_relay::ids::{DeviceId, PairingId, PeerKind};
use dynwork_relay::store::{
    ClaimOutcome, DesktopRegistration, DeviceRecord, NewPairing, PairingOutcome, PhoneClaim,
    RegisterOutcome, RelayStore, RevokeOutcome, RevokedDevice, StoreError,
};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
struct Pairing {
    id: PairingId,
    user: UserId,
    expires_at: DateTime<Utc>,
    claimed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default)]
pub(crate) struct Mem {
    pub(crate) users: HashSet<UserId>,
    pub(crate) inactive: HashSet<UserId>,
    pub(crate) devices: Vec<(DeviceRecord, Option<CredentialHash>)>,
    pairings: HashMap<CredentialHash, Pairing>,
    pub(crate) grants: Vec<crate::memory_mobile::Grant>,
}

impl Mem {
    pub(crate) fn live(&self, user: &UserId) -> impl Iterator<Item = &DeviceRecord> {
        self.devices
            .iter()
            .map(|(device, _)| device)
            .filter(move |device| &device.user == user && device.revoked_at.is_none())
    }
}

#[derive(Debug, Default)]
pub(crate) struct MemoryStore {
    pub(crate) mem: Mutex<Mem>,
}

impl MemoryStore {
    pub(crate) fn add_user(&self, user: &UserId) {
        self.mem.lock().unwrap().users.insert(user.clone());
    }

    // Revokes behind the routes' back, as an operator or another flow would.
    pub(crate) fn revoke_directly(&self, device: DeviceId, at: DateTime<Utc>) {
        let mut mem = self.mem.lock().unwrap();
        for (record, _) in &mut mem.devices {
            if record.id == device {
                record.revoked_at = Some(at);
            }
        }
    }
}

#[async_trait]
impl RelayStore for MemoryStore {
    async fn register_desktop(
        &self,
        registration: &DesktopRegistration,
    ) -> Result<RegisterOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        if !mem.users.contains(&registration.user) {
            return Ok(RegisterOutcome::UnknownUser);
        }
        if let Some(existing) = mem
            .live(&registration.user)
            .find(|device| device.peer == registration.peer)
            .cloned()
        {
            return Ok(if existing.kind == PeerKind::Desktop {
                RegisterOutcome::Registered(existing)
            } else {
                RegisterOutcome::PeerTaken
            });
        }
        if mem.live(&registration.user).count() >= registration.max_devices as usize {
            return Ok(RegisterOutcome::DeviceLimit);
        }
        let device = DeviceRecord {
            id: DeviceId::generate(),
            user: registration.user.clone(),
            kind: PeerKind::Desktop,
            peer: registration.peer,
            name: String::new(),
            created_at: registration.now,
            last_seen_at: Some(registration.now),
            revoked_at: None,
        };
        mem.devices.push((device.clone(), None));
        Ok(RegisterOutcome::Registered(device))
    }

    async fn count_open_pairings(
        &self,
        user: &UserId,
        now: DateTime<Utc>,
    ) -> Result<i64, StoreError> {
        let mem = self.mem.lock().unwrap();
        Ok(mem
            .pairings
            .values()
            .filter(|p| &p.user == user && p.claimed_at.is_none() && p.expires_at > now)
            .count() as i64)
    }

    async fn insert_pairing(
        &self,
        pairing: &NewPairing,
        max_open: u32,
    ) -> Result<PairingOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        if !mem.users.contains(&pairing.user) {
            return Ok(PairingOutcome::UnknownUser);
        }
        let open = mem
            .pairings
            .values()
            .filter(|p| {
                p.user == pairing.user
                    && p.claimed_at.is_none()
                    && p.expires_at > pairing.created_at
            })
            .count();
        if open >= max_open as usize {
            return Ok(PairingOutcome::TooMany);
        }
        mem.pairings.insert(
            pairing.token.clone(),
            Pairing {
                id: pairing.id,
                user: pairing.user.clone(),
                expires_at: pairing.expires_at,
                claimed_at: None,
            },
        );
        Ok(PairingOutcome::Inserted)
    }

    async fn claim_pairing(&self, claim: &PhoneClaim) -> Result<ClaimOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        let Some(pairing) = mem.pairings.get(&claim.token).cloned() else {
            return Ok(ClaimOutcome::Unknown);
        };
        if pairing.claimed_at.is_some() {
            return Ok(ClaimOutcome::Used);
        }
        if pairing.expires_at <= claim.now {
            return Ok(ClaimOutcome::Expired);
        }
        let user = pairing.user.clone();
        if mem.live(&user).any(|device| device.peer == claim.peer) {
            return Ok(ClaimOutcome::PeerTaken);
        }
        if mem.live(&user).count() >= claim.max_devices as usize {
            return Ok(ClaimOutcome::DeviceLimit);
        }
        let device = DeviceRecord {
            id: claim.device,
            user,
            kind: PeerKind::Phone,
            peer: claim.peer,
            name: claim.name.clone(),
            created_at: claim.now,
            last_seen_at: Some(claim.now),
            revoked_at: None,
        };
        mem.devices
            .push((device.clone(), Some(claim.credential.clone())));
        if let Some(row) = mem.pairings.get_mut(&claim.token) {
            row.claimed_at = Some(claim.now);
        }
        Ok(ClaimOutcome::Claimed {
            device,
            pairing: pairing.id,
        })
    }

    async fn find_phone_by_credential(
        &self,
        credential: &CredentialHash,
    ) -> Result<Option<DeviceRecord>, StoreError> {
        let mem = self.mem.lock().unwrap();
        Ok(mem
            .devices
            .iter()
            .find(|(device, hash)| hash.as_ref() == Some(credential) && device.revoked_at.is_none())
            .map(|(device, _)| device.clone()))
    }

    async fn list_devices(&self, user: &UserId) -> Result<Vec<DeviceRecord>, StoreError> {
        Ok(self.mem.lock().unwrap().live(user).cloned().collect())
    }

    async fn revoke_device(
        &self,
        user: &UserId,
        device: DeviceId,
        now: DateTime<Utc>,
    ) -> Result<RevokeOutcome, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        for (record, _) in &mut mem.devices {
            if record.id == device && &record.user == user && record.revoked_at.is_none() {
                if record.kind == PeerKind::Desktop {
                    return Ok(RevokeOutcome::Desktop);
                }
                record.revoked_at = Some(now);
                return Ok(RevokeOutcome::Revoked);
            }
        }
        Ok(RevokeOutcome::NotFound)
    }

    async fn list_revoked_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<RevokedDevice>, StoreError> {
        let mem = self.mem.lock().unwrap();
        Ok(mem
            .devices
            .iter()
            .filter_map(|(device, _)| {
                device
                    .revoked_at
                    .filter(|at| *at > since)
                    .map(|revoked_at| RevokedDevice {
                        device: device.id,
                        user: device.user.clone(),
                        revoked_at,
                    })
            })
            .collect())
    }

    async fn touch_device(&self, device: DeviceId, now: DateTime<Utc>) -> Result<(), StoreError> {
        let mut mem = self.mem.lock().unwrap();
        for (record, _) in &mut mem.devices {
            if record.id == device {
                record.last_seen_at = Some(now);
            }
        }
        Ok(())
    }

    async fn delete_expired_pairings(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        let mut mem = self.mem.lock().unwrap();
        let start = mem.pairings.len();
        mem.pairings.retain(|_, p| p.expires_at >= before);
        Ok((start - mem.pairings.len()) as u64)
    }
}

//! [`RelayStore`] over Postgres, with compile-time checked queries.
//!
//! Device creation locks the owner's `users` row for the transaction, so two
//! concurrent pairings cannot both slip under the per-user device limit, and
//! a claim locks its pairing row, so a token is redeemed once.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::rows::{NewDevice, count_live_devices, find_live_peer, insert_device, lock_user};
use super::types::{DeviceRow, corrupt};
use super::{
    ClaimOutcome, DesktopRegistration, DeviceRecord, NewPairing, PairingOutcome, PhoneClaim,
    RegisterOutcome, RelayStore, RevokeOutcome, RevokedDevice, StoreError,
};
use crate::credential::CredentialHash;
use crate::ids::{DeviceId, PairingId, PeerKind};

#[derive(Debug, Clone)]
pub struct PgRelayStore {
    pub(super) pool: Arc<PgPool>,
}

impl PgRelayStore {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RelayStore for PgRelayStore {
    async fn register_desktop(
        &self,
        registration: &DesktopRegistration,
    ) -> Result<RegisterOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, &registration.user).await? {
            return Ok(RegisterOutcome::UnknownUser);
        }
        if let Some(existing) =
            find_live_peer(&mut tx, &registration.user, registration.peer).await?
        {
            if existing.kind != PeerKind::Desktop {
                return Ok(RegisterOutcome::PeerTaken);
            }
            sqlx::query!(
                "UPDATE dynwork_relay_devices SET last_seen_at = $2 WHERE id = $1",
                existing.id.to_string(),
                registration.now
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(RegisterOutcome::Registered(existing));
        }
        if count_live_devices(&mut tx, &registration.user).await?
            >= i64::from(registration.max_devices)
        {
            return Ok(RegisterOutcome::DeviceLimit);
        }
        let device = insert_device(
            &mut tx,
            &NewDevice {
                id: DeviceId::generate(),
                user: &registration.user,
                kind: PeerKind::Desktop,
                peer: registration.peer,
                name: "",
                credential: None,
                now: registration.now,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(RegisterOutcome::Registered(device))
    }

    async fn count_open_pairings(
        &self,
        user: &UserId,
        now: DateTime<Utc>,
    ) -> Result<i64, StoreError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM dynwork_relay_pairings
             WHERE user_id = $1 AND claimed_at IS NULL AND expires_at > $2",
            user.as_str(),
            now
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(count.unwrap_or(0))
    }

    async fn insert_pairing(
        &self,
        pairing: &NewPairing,
        max_open: u32,
    ) -> Result<PairingOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, &pairing.user).await? {
            return Ok(PairingOutcome::UnknownUser);
        }
        let open = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM dynwork_relay_pairings
             WHERE user_id = $1 AND claimed_at IS NULL AND expires_at > $2",
            pairing.user.as_str(),
            pairing.created_at
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);
        if open >= i64::from(max_open) {
            return Ok(PairingOutcome::TooMany);
        }
        sqlx::query!(
            "INSERT INTO dynwork_relay_pairings (id, user_id, token_hash, created_at, expires_at)
             VALUES ($1, $2, $3, $4, $5)",
            pairing.id.to_string(),
            pairing.user.as_str(),
            pairing.token.as_str(),
            pairing.created_at,
            pairing.expires_at
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(PairingOutcome::Inserted)
    }

    async fn claim_pairing(&self, claim: &PhoneClaim) -> Result<ClaimOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        let pairing = sqlx::query!(
            "SELECT id, user_id, expires_at, claimed_at FROM dynwork_relay_pairings
             WHERE token_hash = $1 FOR UPDATE",
            claim.token.as_str()
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(pairing) = pairing else {
            return Ok(ClaimOutcome::Unknown);
        };
        if pairing.claimed_at.is_some() {
            return Ok(ClaimOutcome::Used);
        }
        if pairing.expires_at <= claim.now {
            return Ok(ClaimOutcome::Expired);
        }
        let id: PairingId = pairing
            .id
            .parse()
            .map_err(corrupt("dynwork_relay_pairings.id"))?;
        let user = UserId::new(pairing.user_id);
        if !lock_user(&mut tx, &user).await? {
            return Ok(ClaimOutcome::Unknown);
        }
        if find_live_peer(&mut tx, &user, claim.peer).await?.is_some() {
            return Ok(ClaimOutcome::PeerTaken);
        }
        if count_live_devices(&mut tx, &user).await? >= i64::from(claim.max_devices) {
            return Ok(ClaimOutcome::DeviceLimit);
        }
        let device = insert_device(
            &mut tx,
            &NewDevice {
                id: claim.device,
                user: &user,
                kind: PeerKind::Phone,
                peer: claim.peer,
                name: &claim.name,
                credential: Some(&claim.credential),
                now: claim.now,
            },
        )
        .await?;
        sqlx::query!(
            "UPDATE dynwork_relay_pairings SET claimed_at = $2, device_id = $3 WHERE id = $1",
            id.to_string(),
            claim.now,
            claim.device.to_string()
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ClaimOutcome::Claimed {
            device,
            pairing: id,
        })
    }

    async fn find_phone_by_credential(
        &self,
        credential: &CredentialHash,
    ) -> Result<Option<DeviceRecord>, StoreError> {
        let row = sqlx::query_as!(
            DeviceRow,
            "SELECT id, user_id AS owner, kind, peer_id, name, created_at, last_seen_at, revoked_at
             FROM dynwork_relay_devices
             WHERE credential_hash = $1 AND kind = 'phone' AND revoked_at IS NULL",
            credential.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;
        row.map(DeviceRecord::try_from).transpose()
    }

    async fn list_devices(&self, user: &UserId) -> Result<Vec<DeviceRecord>, StoreError> {
        let rows = sqlx::query_as!(
            DeviceRow,
            "SELECT id, user_id AS owner, kind, peer_id, name, created_at, last_seen_at, revoked_at
             FROM dynwork_relay_devices
             WHERE user_id = $1 AND revoked_at IS NULL
             ORDER BY created_at, id",
            user.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.into_iter().map(DeviceRecord::try_from).collect()
    }

    async fn revoke_device(
        &self,
        user: &UserId,
        device: DeviceId,
        now: DateTime<Utc>,
    ) -> Result<RevokeOutcome, StoreError> {
        let done = sqlx::query!(
            "UPDATE dynwork_relay_devices SET revoked_at = $3
             WHERE id = $1 AND user_id = $2 AND kind = 'phone' AND revoked_at IS NULL",
            device.to_string(),
            user.as_str(),
            now
        )
        .execute(&*self.pool)
        .await?;
        if done.rows_affected() == 1 {
            return Ok(RevokeOutcome::Revoked);
        }
        let desktop = sqlx::query_scalar!(
            "SELECT id FROM dynwork_relay_devices
             WHERE id = $1 AND user_id = $2 AND kind = 'desktop' AND revoked_at IS NULL",
            device.to_string(),
            user.as_str()
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(if desktop.is_some() {
            RevokeOutcome::Desktop
        } else {
            RevokeOutcome::NotFound
        })
    }

    async fn list_revoked_since(
        &self,
        since: DateTime<Utc>,
    ) -> Result<Vec<RevokedDevice>, StoreError> {
        let rows = sqlx::query!(
            "SELECT id, user_id, revoked_at FROM dynwork_relay_devices
             WHERE revoked_at > $1 ORDER BY revoked_at",
            since
        )
        .fetch_all(&*self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(RevokedDevice {
                    device: row
                        .id
                        .parse()
                        .map_err(corrupt("dynwork_relay_devices.id"))?,
                    user: UserId::new(row.user_id),
                    revoked_at: row
                        .revoked_at
                        .ok_or(StoreError::Corrupt("dynwork_relay_devices.revoked_at"))?,
                })
            })
            .collect()
    }

    async fn touch_device(&self, device: DeviceId, now: DateTime<Utc>) -> Result<(), StoreError> {
        sqlx::query!(
            "UPDATE dynwork_relay_devices SET last_seen_at = $2 WHERE id = $1",
            device.to_string(),
            now
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    async fn delete_expired_pairings(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        let done = sqlx::query!(
            "DELETE FROM dynwork_relay_pairings WHERE expires_at < $1",
            before
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }
}

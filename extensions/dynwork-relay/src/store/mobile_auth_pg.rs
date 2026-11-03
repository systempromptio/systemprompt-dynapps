//! [`MobileAuthStore`] over Postgres.
//!
//! Each step locks its grant row (`FOR UPDATE`), so two requests racing on the
//! same consent or the same code see one outcome each. Redeeming locks the
//! owner's `users` row as device creation does elsewhere, so the device
//! limit holds against a concurrent pairing claim.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgExecutor, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use super::StoreError;
use super::mobile_auth::{
    Account, CodeRedemption, DecisionOutcome, GrantDecision, GrantOutcome, MobileAuthStore,
    NewGrant, RedeemOutcome,
};
use super::postgres::PgRelayStore;
use super::rows::{NewDevice, count_live_devices, find_live_peer, insert_device, lock_user};
use super::types::corrupt;
use crate::ids::{DeviceId, PeerKind};

async fn select_account(
    executor: impl PgExecutor<'_>,
    user: &UserId,
) -> Result<Option<Account>, StoreError> {
    let row = sqlx::query!(
        "SELECT id, name, email, roles FROM users
         WHERE id = $1 AND status = 'active' AND account_kind = 'person'",
        user.as_str()
    )
    .fetch_optional(executor)
    .await?;
    Ok(row.map(|row| Account {
        user: UserId::new(row.id),
        name: row.name,
        email: row.email,
        roles: row.roles,
    }))
}

async fn revoke_live(
    tx: &mut Transaction<'_, Postgres>,
    device: DeviceId,
    now: DateTime<Utc>,
) -> Result<bool, StoreError> {
    let done = sqlx::query!(
        "UPDATE dynwork_relay_devices SET revoked_at = $2 WHERE id = $1 AND revoked_at IS NULL",
        device.to_string(),
        now
    )
    .execute(&mut **tx)
    .await?;
    Ok(done.rows_affected() == 1)
}

#[async_trait]
impl MobileAuthStore for PgRelayStore {
    async fn insert_grant(&self, grant: &NewGrant) -> Result<GrantOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        if !lock_user(&mut tx, &grant.user).await? {
            return Ok(GrantOutcome::UnknownUser);
        }
        let open = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM dynwork_relay_mobile_grants
             WHERE user_id = $1 AND decided_at IS NULL AND expires_at > $2",
            grant.user.as_str(),
            grant.now
        )
        .fetch_one(&mut *tx)
        .await?
        .unwrap_or(0);
        if open >= i64::from(grant.max_open) {
            return Ok(GrantOutcome::TooMany);
        }
        sqlx::query!(
            "INSERT INTO dynwork_relay_mobile_grants
                 (id, user_id, request_hash, code_challenge, state, device_name, created_at, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            grant.id.to_string(),
            grant.user.as_str(),
            grant.request.as_str(),
            grant.challenge,
            grant.state,
            grant.device_name,
            grant.now,
            grant.expires_at
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(GrantOutcome::Inserted)
    }

    async fn decide_grant(&self, decision: &GrantDecision) -> Result<DecisionOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        let grant = sqlx::query!(
            "SELECT id, user_id, state, expires_at, decided_at FROM dynwork_relay_mobile_grants
             WHERE request_hash = $1 FOR UPDATE",
            decision.request.as_str()
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(grant) = grant.filter(|grant| grant.user_id == decision.user.as_str()) else {
            return Ok(DecisionOutcome::Unknown);
        };
        if grant.decided_at.is_some() {
            return Ok(DecisionOutcome::Decided);
        }
        if grant.expires_at <= decision.now {
            return Ok(DecisionOutcome::Expired);
        }
        let (code, expires_at) = decision
            .code
            .as_ref()
            .map_or((None, grant.expires_at), |(hash, at)| {
                (Some(hash.as_str()), *at)
            });
        sqlx::query!(
            "UPDATE dynwork_relay_mobile_grants
             SET decided_at = $2, approved = $3, code_hash = $4, expires_at = $5
             WHERE id = $1",
            grant.id,
            decision.now,
            code.is_some(),
            code,
            expires_at
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(if code.is_some() {
            DecisionOutcome::Approved { state: grant.state }
        } else {
            DecisionOutcome::Denied { state: grant.state }
        })
    }

    async fn redeem_code(&self, redemption: &CodeRedemption) -> Result<RedeemOutcome, StoreError> {
        let mut tx = self.pool.begin().await?;
        let grant = sqlx::query!(
            "SELECT id, user_id, code_challenge, device_name, expires_at, redeemed_at, device_id
             FROM dynwork_relay_mobile_grants WHERE code_hash = $1 FOR UPDATE",
            redemption.code.as_str()
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(grant) = grant else {
            return Ok(RedeemOutcome::Unknown);
        };
        if grant.redeemed_at.is_some() {
            let first: Option<DeviceId> = grant
                .device_id
                .map(|id| {
                    id.parse()
                        .map_err(corrupt("dynwork_relay_mobile_grants.device_id"))
                })
                .transpose()?;
            let revoked = match first {
                Some(device) if revoke_live(&mut tx, device, redemption.now).await? => Some(device),
                _ => None,
            };
            tx.commit().await?;
            return Ok(RedeemOutcome::Used {
                user: UserId::new(grant.user_id),
                revoked,
            });
        }
        sqlx::query!(
            "UPDATE dynwork_relay_mobile_grants SET redeemed_at = $2 WHERE id = $1",
            grant.id,
            redemption.now
        )
        .execute(&mut *tx)
        .await?;
        let user = UserId::new(grant.user_id);
        let outcome = if grant.expires_at <= redemption.now {
            RedeemOutcome::Expired
        } else if grant.code_challenge != redemption.challenge {
            RedeemOutcome::Mismatch
        } else if !lock_user(&mut tx, &user).await? {
            RedeemOutcome::UnknownUser
        } else {
            sign_in(&mut tx, &grant.id, &user, &grant.device_name, redemption).await?
        };
        tx.commit().await?;
        Ok(outcome)
    }

    async fn find_account(&self, user: &UserId) -> Result<Option<Account>, StoreError> {
        select_account(&*self.pool, user).await
    }

    async fn delete_expired_grants(&self, before: DateTime<Utc>) -> Result<u64, StoreError> {
        let done = sqlx::query!(
            "DELETE FROM dynwork_relay_mobile_grants WHERE expires_at < $1",
            before
        )
        .execute(&*self.pool)
        .await?;
        Ok(done.rows_affected())
    }
}

async fn sign_in(
    tx: &mut Transaction<'_, Postgres>,
    grant: &str,
    user: &UserId,
    name: &str,
    redemption: &CodeRedemption,
) -> Result<RedeemOutcome, StoreError> {
    let Some(account) = select_account(&mut **tx, user).await? else {
        return Ok(RedeemOutcome::UnknownUser);
    };
    let replaced = match find_live_peer(tx, user, redemption.peer).await? {
        Some(existing) if existing.kind == PeerKind::Desktop => {
            return Ok(RedeemOutcome::PeerTaken);
        },
        existing => existing.map(|device| device.id),
    };
    let live = count_live_devices(tx, user).await? - i64::from(replaced.is_some());
    if live >= i64::from(redemption.max_devices) {
        return Ok(RedeemOutcome::DeviceLimit);
    }
    if let Some(previous) = replaced {
        revoke_live(tx, previous, redemption.now).await?;
    }
    let device = insert_device(
        tx,
        &NewDevice {
            id: redemption.device,
            user,
            kind: PeerKind::Phone,
            peer: redemption.peer,
            name,
            credential: Some(&redemption.credential),
            now: redemption.now,
        },
    )
    .await?;
    sqlx::query!(
        "UPDATE dynwork_relay_mobile_grants SET device_id = $2 WHERE id = $1",
        grant,
        redemption.device.to_string()
    )
    .execute(&mut **tx)
    .await?;
    Ok(RedeemOutcome::SignedIn {
        device,
        account,
        replaced,
    })
}

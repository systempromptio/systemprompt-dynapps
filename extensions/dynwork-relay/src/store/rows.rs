//! Row mapping and the transaction steps [`super::postgres::PgRelayStore`]
//! composes: lock the owner, count live devices, find a live peer, insert a
//! device.

use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};
use systemprompt::identifiers::UserId;

use super::types::DeviceRow;
use super::{DeviceRecord, StoreError};
use crate::credential::CredentialHash;
use crate::ids::{DeviceId, PeerId, PeerKind};

pub(super) struct NewDevice<'a> {
    pub(super) id: DeviceId,
    pub(super) user: &'a UserId,
    pub(super) kind: PeerKind,
    pub(super) peer: PeerId,
    pub(super) name: &'a str,
    pub(super) credential: Option<&'a CredentialHash>,
    pub(super) now: DateTime<Utc>,
}

pub(super) async fn lock_user(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
) -> Result<bool, StoreError> {
    let row = sqlx::query_scalar!(
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        user.as_str()
    )
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.is_some())
}

pub(super) async fn count_live_devices(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
) -> Result<i64, StoreError> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM dynwork_relay_devices WHERE user_id = $1 AND revoked_at IS NULL",
        user.as_str()
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(count.unwrap_or(0))
}

pub(super) async fn find_live_peer(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    peer: PeerId,
) -> Result<Option<DeviceRecord>, StoreError> {
    let row = sqlx::query_as!(
        DeviceRow,
        "SELECT id, user_id AS owner, kind, peer_id, name, created_at, last_seen_at, revoked_at
         FROM dynwork_relay_devices
         WHERE user_id = $1 AND peer_id = $2 AND revoked_at IS NULL",
        user.as_str(),
        peer.to_hex()
    )
    .fetch_optional(&mut **tx)
    .await?;
    row.map(DeviceRecord::try_from).transpose()
}

pub(super) async fn insert_device(
    tx: &mut Transaction<'_, Postgres>,
    device: &NewDevice<'_>,
) -> Result<DeviceRecord, StoreError> {
    sqlx::query!(
        "INSERT INTO dynwork_relay_devices
             (id, user_id, kind, peer_id, name, credential_hash, created_at, last_seen_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $7)",
        device.id.to_string(),
        device.user.as_str(),
        device.kind.as_str(),
        device.peer.to_hex(),
        device.name,
        device.credential.map(CredentialHash::as_str),
        device.now
    )
    .execute(&mut **tx)
    .await?;
    Ok(DeviceRecord {
        id: device.id,
        user: device.user.clone(),
        kind: device.kind,
        peer: device.peer,
        name: device.name.to_owned(),
        created_at: device.now,
        last_seen_at: Some(device.now),
        revoked_at: None,
    })
}

//! Row shapes as the queries return them, before conversion into the
//! store's typed records.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use super::{DeviceRecord, StoreError};
use crate::ids::{IdError, PeerId};

pub(super) struct DeviceRow {
    pub(super) id: String,
    pub(super) owner: String,
    pub(super) kind: String,
    pub(super) peer_id: String,
    pub(super) name: String,
    pub(super) created_at: DateTime<Utc>,
    pub(super) last_seen_at: Option<DateTime<Utc>>,
    pub(super) revoked_at: Option<DateTime<Utc>>,
}

impl TryFrom<DeviceRow> for DeviceRecord {
    type Error = StoreError;

    fn try_from(row: DeviceRow) -> Result<Self, StoreError> {
        Ok(Self {
            id: row
                .id
                .parse()
                .map_err(corrupt("dynwork_relay_devices.id"))?,
            user: UserId::new(row.owner),
            kind: row
                .kind
                .parse()
                .map_err(corrupt("dynwork_relay_devices.kind"))?,
            peer: PeerId::parse_hex(&row.peer_id)
                .map_err(corrupt("dynwork_relay_devices.peer_id"))?,
            name: row.name,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
            revoked_at: row.revoked_at,
        })
    }
}

pub(super) fn corrupt(column: &'static str) -> impl FnOnce(IdError) -> StoreError {
    move |error| {
        tracing::error!(%error, column, "dynwork relay row is invalid");
        StoreError::Corrupt(column)
    }
}

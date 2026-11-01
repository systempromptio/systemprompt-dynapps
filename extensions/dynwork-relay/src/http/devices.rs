//! `GET /devices` and `DELETE /devices/{id}`, for the signed-in desktop.
//!
//! Only phones are revoked here. A desktop answers `409
//! desktop_not_revocable`: it signs back in with its `DynWork` JWT, so
//! removing one is an account sign-out.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::audit::AuditEvent;
use crate::auth::{bearer, verify_desktop};
use crate::error::RelayError;
use crate::ids::{DeviceId, PeerId, PeerKind};
use crate::state::RelayState;
use crate::store::RevokeOutcome;

#[derive(Debug, Serialize)]
pub struct DeviceView {
    pub id: DeviceId,
    pub kind: PeerKind,
    pub peer_id: PeerId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub online: bool,
}

pub async fn list(
    State(state): State<RelayState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DeviceView>>, RelayError> {
    let desktop = verify_desktop(state.authority.as_ref(), bearer(&headers)?)?;
    let devices = state.store.list_devices(&desktop.user).await?;
    Ok(Json(
        devices
            .into_iter()
            .map(|device| DeviceView {
                online: state.hub.is_online(device.id),
                id: device.id,
                kind: device.kind,
                peer_id: device.peer,
                name: device.name,
                created_at: device.created_at,
                last_seen_at: device.last_seen_at,
            })
            .collect(),
    ))
}

pub async fn revoke(
    State(state): State<RelayState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, RelayError> {
    let desktop = verify_desktop(state.authority.as_ref(), bearer(&headers)?)?;
    let device: DeviceId = id.parse().map_err(|error: crate::ids::IdError| {
        tracing::debug!(%error, "device id in the path is not a UUID");
        RelayError::NotFound
    })?;
    match state
        .store
        .revoke_device(&desktop.user, device, state.clock.now())
        .await?
    {
        RevokeOutcome::Revoked => {},
        RevokeOutcome::Desktop => return Err(RelayError::DesktopNotRevocable),
        RevokeOutcome::NotFound => return Err(RelayError::NotFound),
    }
    state.hub.revoke(device);
    state.audit.record(&AuditEvent::Revoke {
        user: desktop.user,
        device,
    });
    Ok(StatusCode::NO_CONTENT)
}

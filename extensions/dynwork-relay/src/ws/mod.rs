//! `GET /desktop` and `GET /device`: authenticate, then upgrade.
//!
//! Authentication happens before the upgrade, so a refused socket is a plain
//! HTTP 401 and never holds a tunnel. Each tunnel takes a permit from the
//! per-instance tunnel limit and runs on core's `BackgroundTasks`, so a
//! graceful shutdown closes it with a reconnect hint and waits for it.

mod session;

use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::{DateTime, Utc};
use tokio::sync::{OwnedSemaphorePermit, TryAcquireError};

use crate::auth::device::verify_device;
use crate::auth::{AuthError, bearer, verify_desktop};
use crate::error::RelayError;
use crate::hub::Endpoint;
use crate::ids::{PeerId, PeerKind};
use crate::state::RelayState;
use crate::store::{DesktopRegistration, RegisterOutcome};

pub const PEER_HEADER: &str = "x-dynwork-peer";

fn permit(state: &RelayState) -> Result<OwnedSemaphorePermit, RelayError> {
    std::sync::Arc::clone(&state.tunnels)
        .try_acquire_owned()
        .map_err(|_full: TryAcquireError| RelayError::Overloaded)
}

fn lifetime(state: &RelayState, expires_at: Option<DateTime<Utc>>) -> Result<Duration, RelayError> {
    let cap = state.limits.session_lifetime;
    let Some(at) = expires_at else {
        return Ok(cap);
    };
    match (at - state.clock.now()).to_std() {
        Ok(left) if !left.is_zero() => Ok(left.min(cap)),
        _ => Err(AuthError::Expired.into()),
    }
}

pub async fn desktop_socket(
    State(state): State<RelayState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, RelayError> {
    let desktop = verify_desktop(state.authority.as_ref(), bearer(&headers)?)?;
    let peer = headers
        .get(PEER_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(RelayError::BadRequest(
            "the x-dynwork-peer header is required",
        ))
        .and_then(|text| {
            PeerId::parse_hex(text).map_err(|error| {
                tracing::debug!(%error, "x-dynwork-peer is not a peer id");
                RelayError::BadRequest("x-dynwork-peer is 32 hex digits")
            })
        })?;
    let lifetime = lifetime(&state, Some(desktop.expires_at))?;
    let permit = permit(&state)?;
    let registration = DesktopRegistration {
        user: desktop.user.clone(),
        peer,
        now: state.clock.now(),
        max_devices: state.limits.devices_per_user,
    };
    let device = match state.store.register_desktop(&registration).await? {
        RegisterOutcome::Registered(device) => device,
        RegisterOutcome::DeviceLimit => return Err(RelayError::DeviceLimit),
        RegisterOutcome::PeerTaken => return Err(RelayError::PeerTaken),
        RegisterOutcome::UnknownUser => return Err(AuthError::UnknownUser.into()),
    };
    let endpoint = Endpoint {
        user: desktop.user,
        peer,
        device: device.id,
        kind: PeerKind::Desktop,
    };
    Ok(upgrade(ws, state, endpoint, lifetime, permit))
}

pub async fn device_socket(
    State(state): State<RelayState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, RelayError> {
    let device = verify_device(&state.device_verifiers, bearer(&headers)?).await?;
    let lifetime = lifetime(&state, device.expires_at)?;
    let permit = permit(&state)?;
    state
        .store
        .touch_device(device.device, state.clock.now())
        .await?;
    let endpoint = Endpoint {
        user: device.user,
        peer: device.peer,
        device: device.device,
        kind: PeerKind::Phone,
    };
    Ok(upgrade(ws, state, endpoint, lifetime, permit))
}

fn upgrade(
    ws: WebSocketUpgrade,
    state: RelayState,
    endpoint: Endpoint,
    lifetime: Duration,
    permit: OwnedSemaphorePermit,
) -> Response {
    let transport_limit = state.limits.max_envelope.saturating_mul(2);
    ws.max_message_size(transport_limit)
        .max_frame_size(transport_limit)
        .on_upgrade(move |socket| async move {
            let tasks = state.tasks.clone();
            let tunnel = session::Tunnel {
                state,
                endpoint,
                lifetime,
                permit,
            };
            tasks.spawn_cancellable("dynwork_relay_tunnel", move |shutdown| {
                session::run(socket, tunnel, shutdown)
            });
        })
}

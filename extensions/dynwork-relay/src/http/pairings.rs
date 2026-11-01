//! `POST /pairings` (desktop) and `POST /pairings/claim` (phone).
//!
//! The pairing token travels in the JSON body, never in the URI, so no access
//! log or proxy records it.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::audit::AuditEvent;
use crate::auth::{AuthError, bearer, verify_desktop};
use crate::credential::{CredentialHash, DeviceCredential, PairingToken};
use crate::error::RelayError;
use crate::ids::{DeviceId, PairingId, PeerId};
use crate::state::RelayState;
use crate::store::{ClaimOutcome, NewPairing, PairingOutcome, PhoneClaim};

const MAX_NAME_CHARS: usize = 64;
const DEFAULT_NAME: &str = "Phone";

#[derive(Debug, Serialize)]
pub struct PairingCreated {
    pub pairing_id: PairingId,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn create(
    State(state): State<RelayState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<PairingCreated>), RelayError> {
    let desktop = verify_desktop(state.authority.as_ref(), bearer(&headers)?)?;
    let now = state.clock.now();
    let ttl = chrono::Duration::from_std(state.limits.pairing_ttl).map_err(|error| {
        tracing::error!(%error, "dynwork relay pairing ttl out of range");
        RelayError::BadRequest("pairing ttl out of range")
    })?;
    let (token, hash) = PairingToken::mint();
    let pairing = NewPairing {
        id: PairingId::generate(),
        user: desktop.user,
        token: hash,
        created_at: now,
        expires_at: now + ttl,
    };
    match state
        .store
        .insert_pairing(&pairing, state.limits.open_pairings_per_user)
        .await?
    {
        PairingOutcome::Inserted => {},
        PairingOutcome::TooMany => return Err(RelayError::TooManyPairings),
        PairingOutcome::UnknownUser => return Err(AuthError::UnknownUser.into()),
    }
    state.audit.record(&AuditEvent::PairingCreated {
        user: pairing.user.clone(),
        pairing: pairing.id,
    });
    Ok((
        StatusCode::CREATED,
        Json(PairingCreated {
            pairing_id: pairing.id,
            token: token.expose().to_owned(),
            expires_at: pairing.expires_at,
        }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRequest {
    pub token: String,
    pub peer_id: PeerId,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Claimed {
    pub device_id: DeviceId,
    pub peer_id: PeerId,
    pub credential: String,
}

fn device_name(raw: &str) -> Result<String, RelayError> {
    let name = raw.trim();
    if name.chars().any(char::is_control) || name.chars().count() > MAX_NAME_CHARS {
        return Err(RelayError::BadRequest(
            "a device name is at most 64 printable characters",
        ));
    }
    Ok(if name.is_empty() {
        DEFAULT_NAME.to_owned()
    } else {
        name.to_owned()
    })
}

pub async fn claim(
    State(state): State<RelayState>,
    Json(request): Json<ClaimRequest>,
) -> Result<(StatusCode, Json<Claimed>), RelayError> {
    let token =
        CredentialHash::of_pairing_token(&request.token).ok_or(RelayError::PairingUnknown)?;
    let name = device_name(&request.name)?;
    let (credential, hash) = DeviceCredential::mint();
    let claim = PhoneClaim {
        token,
        device: DeviceId::generate(),
        peer: request.peer_id,
        name,
        credential: hash,
        now: state.clock.now(),
        max_devices: state.limits.devices_per_user,
    };
    let (device, pairing) = match state.store.claim_pairing(&claim).await? {
        ClaimOutcome::Claimed { device, pairing } => (device, pairing),
        ClaimOutcome::Unknown => return Err(RelayError::PairingUnknown),
        ClaimOutcome::Used => return Err(RelayError::PairingUsed),
        ClaimOutcome::Expired => return Err(RelayError::PairingExpired),
        ClaimOutcome::DeviceLimit => return Err(RelayError::DeviceLimit),
        ClaimOutcome::PeerTaken => return Err(RelayError::PeerTaken),
    };
    state.audit.record(&AuditEvent::Pair {
        user: device.user.clone(),
        pairing,
        device: device.id,
    });
    Ok((
        StatusCode::CREATED,
        Json(Claimed {
            device_id: device.id,
            peer_id: device.peer,
            credential: credential.expose().to_owned(),
        }),
    ))
}

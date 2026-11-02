//! The phone's JSON routes: discovery, the code exchange, JWT refresh, the
//! session check and sign-out.
//!
//! The exchange takes the one-time code, its PKCE verifier and the phone's
//! peer id, and answers with a mobile credential (`dwm1_`, kept by the relay
//! only as its SHA-256) and a one-hour JWT. Every failure of the code itself
//! (unknown, used, expired, wrong verifier) is the same `invalid_grant`, so
//! the answer says nothing about which. The credential refreshes the JWT, opens
//! the device socket, and signs the phone out; revoking the device from the
//! devices list ends all three.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use systemprompt::identifiers::UserId;

use super::jwt::{self, MobileToken};
use super::params::{challenge_of, valid_verifier};
use super::{MOBILE_BASE_PATH, REDIRECT_URI};
use crate::audit::AuditEvent;
use crate::auth::{AuthError, bearer};
use crate::credential::{CredentialHash, MobileCredential};
use crate::error::RelayError;
use crate::ids::{DeviceId, PeerId};
use crate::router::BASE_PATH;
use crate::state::RelayState;
use crate::store::{Account, CodeRedemption, DeviceRecord, RedeemOutcome};

#[derive(Debug, Serialize)]
pub struct AccountView {
    pub id: String,
    pub email: String,
    pub name: String,
}

impl From<&Account> for AccountView {
    fn from(account: &Account) -> Self {
        Self {
            id: account.user.to_string(),
            email: account.email.clone(),
            name: account.name.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionAnswer {
    pub token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub device_id: DeviceId,
    pub account: AccountView,
}

#[derive(Debug, Serialize)]
pub struct SignedIn {
    pub credential: String,
    pub peer_id: PeerId,
    #[serde(flatten)]
    pub session: SessionAnswer,
}

fn no_store(status: StatusCode, body: impl Serialize) -> Response {
    (
        status,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
        ],
        Json(body),
    )
        .into_response()
}

fn session(token: MobileToken, device: DeviceId, account: &Account) -> SessionAnswer {
    SessionAnswer {
        token: token.token,
        token_type: "Bearer",
        expires_in: token.expires_in,
        device_id: device,
        account: account.into(),
    }
}

fn mint(
    state: &RelayState,
    account: &Account,
    device: DeviceId,
) -> Result<MobileToken, RelayError> {
    jwt::mint(state.authority.as_ref(), account, device).map_err(|error| {
        tracing::error!(%error, "dynwork mobile: JWT mint failed");
        RelayError::Signing
    })
}

pub async fn discovery(State(state): State<RelayState>) -> Json<serde_json::Value> {
    Json(json!({
        "version": 1,
        "display_name": state.branding.display_name,
        "redirect_uri": REDIRECT_URI,
        "authorize_path": format!("{MOBILE_BASE_PATH}/authorize"),
        "token_path": format!("{MOBILE_BASE_PATH}/token"),
        "refresh_path": format!("{MOBILE_BASE_PATH}/refresh"),
        "session_path": format!("{MOBILE_BASE_PATH}/session"),
        "revoke_path": format!("{MOBILE_BASE_PATH}/revoke"),
        "device_socket_path": format!("{BASE_PATH}/device"),
        "code_challenge_methods": ["S256"],
    }))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenRequest {
    pub code: String,
    pub code_verifier: String,
    pub peer_id: PeerId,
}

pub async fn exchange(
    State(state): State<RelayState>,
    request: Result<Json<TokenRequest>, JsonRejection>,
) -> Result<Response, RelayError> {
    let Json(request) = request.map_err(|error| {
        tracing::debug!(%error, "dynwork mobile: token request unreadable");
        RelayError::BadRequest("the body is {code, code_verifier, peer_id}")
    })?;
    if !valid_verifier(&request.code_verifier) {
        return Err(RelayError::BadRequest(
            "code_verifier is 43 to 128 unreserved characters",
        ));
    }
    let code = CredentialHash::of_code(&request.code).ok_or(RelayError::InvalidGrant)?;
    let (credential, hash) = MobileCredential::mint();
    let redemption = CodeRedemption {
        code,
        challenge: challenge_of(&request.code_verifier),
        device: DeviceId::generate(),
        peer: request.peer_id,
        credential: hash,
        now: state.clock.now(),
        max_devices: state.limits.devices_per_user,
    };
    let (device, account, replaced) = match state.store.redeem_code(&redemption).await? {
        RedeemOutcome::SignedIn {
            device,
            account,
            replaced,
        } => (device, account, replaced),
        RedeemOutcome::Used { user, revoked } => {
            if let Some(device) = revoked {
                cut(&state, user, device);
            }
            return Err(RelayError::InvalidGrant);
        },
        RedeemOutcome::Unknown | RedeemOutcome::Expired | RedeemOutcome::Mismatch => {
            return Err(RelayError::InvalidGrant);
        },
        RedeemOutcome::UnknownUser => return Err(AuthError::UnknownUser.into()),
        RedeemOutcome::DeviceLimit => return Err(RelayError::DeviceLimit),
        RedeemOutcome::PeerTaken => return Err(RelayError::PeerTaken),
    };
    if let Some(previous) = replaced {
        cut(&state, device.user.clone(), previous);
    }
    state.audit.record(&AuditEvent::MobileSignIn {
        user: device.user.clone(),
        device: device.id,
    });
    let token = mint(&state, &account, device.id)?;
    Ok(no_store(
        StatusCode::OK,
        SignedIn {
            credential: credential.expose().to_owned(),
            peer_id: device.peer,
            session: session(token, device.id, &account),
        },
    ))
}

fn cut(state: &RelayState, user: UserId, device: DeviceId) {
    state.hub.revoke(device);
    state.audit.record(&AuditEvent::Revoke { user, device });
}

async fn signed_in_device(
    state: &RelayState,
    headers: &HeaderMap,
) -> Result<DeviceRecord, RelayError> {
    let hash =
        CredentialHash::of_mobile_credential(bearer(headers)?).ok_or(AuthError::UnknownDevice)?;
    Ok(state
        .store
        .find_phone_by_credential(&hash)
        .await?
        .ok_or(AuthError::UnknownDevice)?)
}

pub async fn refresh(
    State(state): State<RelayState>,
    headers: HeaderMap,
) -> Result<Response, RelayError> {
    let device = signed_in_device(&state, &headers).await?;
    let account = state
        .store
        .find_account(&device.user)
        .await?
        .ok_or(AuthError::UnknownUser)?;
    let token = mint(&state, &account, device.id)?;
    Ok(no_store(
        StatusCode::OK,
        session(token, device.id, &account),
    ))
}

pub async fn current(
    State(state): State<RelayState>,
    headers: HeaderMap,
) -> Result<Response, RelayError> {
    let identity = jwt::verify(state.authority.as_ref(), bearer(&headers)?)?;
    let live = state
        .store
        .list_devices(&identity.user)
        .await?
        .into_iter()
        .any(|device| device.id == identity.device);
    if !live {
        return Err(AuthError::UnknownDevice.into());
    }
    Ok(no_store(
        StatusCode::OK,
        json!({
            "user_id": identity.user.to_string(),
            "email": identity.email,
            "name": identity.name,
            "device_id": identity.device,
            "expires_at": identity.expires_at,
        }),
    ))
}

pub async fn revoke(
    State(state): State<RelayState>,
    headers: HeaderMap,
) -> Result<StatusCode, RelayError> {
    let device = signed_in_device(&state, &headers).await?;
    state
        .store
        .revoke_device(&device.user, device.id, state.clock.now())
        .await?;
    cut(&state, device.user, device.id);
    Ok(StatusCode::NO_CONTENT)
}

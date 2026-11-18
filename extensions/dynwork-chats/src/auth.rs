//! Who is calling a chats route.
//!
//! Two credentials open the routes and nothing else: the `DynWork` desktop JWT
//! (audience `bridge` or `api`, a signed-in person) and the phone's one-hour
//! mobile JWT (audience `dynwork-mobile`). Both are verified by the relay's own
//! functions against the instance's token authority, so the rules stay in one
//! place. The user always comes from the token, never from a body or a path.
//!
//! A mobile JWT names the phone's device row in the relay. That row must still
//! be live: revoking a phone (signing it out, or removing it from the device
//! list) closes its chats at once instead of when its JWT expires.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use dynwork_relay::auth::{DESKTOP_AUDIENCES, bearer, verify_desktop};
use dynwork_relay::ids::{DeviceId, PeerId, PeerKind};
use dynwork_relay::mobile_auth::jwt;
use dynwork_relay::store::postgres::PgRelayStore;
use dynwork_relay::store::{RelayStore, StoreError};
use sqlx::PgPool;
use systemprompt::identifiers::{ClientId, SessionId, UserId};
use systemprompt::models::auth::TokenAuthority;

use crate::error::ChatsError;
use crate::hint::caller_peer;
use crate::state::ChatsState;

/// Which client a caller presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Client {
    Desktop,
    Phone(DeviceId),
}

/// An authenticated caller: the user every row it reads or writes belongs to.
///
/// It also carries the desktop peer it named in `X-Dynwork-Peer`, which its own
/// writes do not send a change hint to, and what the instance's access rules
/// may ask about the credential: the OAuth client the desktop token names and
/// its session. `admission_session` is the session the rules see: the desktop
/// token's own, else one derived from the phone's device id (or, for a desktop
/// token without a session, from the user), so the same caller always presents
/// the same session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller {
    pub user: UserId,
    pub client: Client,
    pub peer: Option<PeerId>,
    pub client_id: Option<ClientId>,
    pub session: Option<SessionId>,
}

const SESSION_SEED: &str = "dynapps:dynwork-chats:session:";

impl Caller {
    pub fn admission_session(&self) -> SessionId {
        if let Some(session) = &self.session {
            return session.clone();
        }
        let seed = match self.client {
            Client::Phone(device) => format!("{SESSION_SEED}phone:{device}"),
            Client::Desktop => format!("{SESSION_SEED}desktop:{}", self.user),
        };
        let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, seed.as_bytes());
        SessionId::new(format!("sess_{id}"))
    }
}

/// Whether a phone's device row is still live.
///
/// Live means not revoked and still the user's. `#[async_trait]` because
/// [`ChatsState`] holds it as `Arc<dyn DeviceCheck>`: the relay's Postgres
/// store in production, a fake in the suites.
#[async_trait]
pub trait DeviceCheck: Send + Sync + Debug {
    async fn phone_is_live(&self, user: &UserId, device: DeviceId) -> Result<bool, StoreError>;
}

/// [`DeviceCheck`] over the relay's own device table.
#[derive(Debug)]
pub struct RelayDevices {
    store: PgRelayStore,
}

impl RelayDevices {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self {
            store: PgRelayStore::new(pool),
        }
    }
}

#[async_trait]
impl DeviceCheck for RelayDevices {
    async fn phone_is_live(&self, user: &UserId, device: DeviceId) -> Result<bool, StoreError> {
        Ok(self
            .store
            .list_devices(user)
            .await?
            .iter()
            .any(|row| row.id == device && row.kind == PeerKind::Phone && row.revoked_at.is_none()))
    }
}

pub async fn authenticate(state: &ChatsState, headers: &HeaderMap) -> Result<Caller, ChatsError> {
    let token = bearer(headers)?;
    let peer = caller_peer(headers);
    let authority = state.authority.as_ref();
    let desktop = match verify_desktop(authority, token) {
        Ok(identity) => {
            return Ok(Caller {
                user: identity.user,
                client: Client::Desktop,
                peer,
                client_id: desktop_client(authority, token),
                session: identity.session,
            });
        },
        Err(refused) => refused,
    };
    let phone = jwt::verify(authority, token).inspect_err(|mobile| {
        tracing::debug!(%desktop, %mobile, "dynwork chats: bearer refused");
    })?;
    if !state
        .devices
        .phone_is_live(&phone.user, phone.device)
        .await?
    {
        tracing::debug!(device = %phone.device, "dynwork chats: phone device is not live");
        return Err(dynwork_relay::auth::AuthError::UnknownDevice.into());
    }
    Ok(Caller {
        user: phone.user,
        client: Client::Phone(phone.device),
        peer,
        client_id: None,
        session: None,
    })
}

fn desktop_client(authority: &dyn TokenAuthority, token: &str) -> Option<ClientId> {
    authority
        .verify_bearer(token, DESKTOP_AUDIENCES)
        .ok()
        .and_then(|claims| claims.custom.client_id)
}

impl FromRequestParts<ChatsState> for Caller {
    type Rejection = ChatsError;

    async fn from_request_parts(parts: &mut Parts, state: &ChatsState) -> Result<Self, ChatsError> {
        authenticate(state, &parts.headers).await
    }
}

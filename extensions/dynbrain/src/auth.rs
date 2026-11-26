//! Who is calling a Brain route.
//!
//! Two credentials open the routes and nothing else: the `DynWork` desktop JWT
//! (audience `bridge` or `api`, a signed-in person) and the phone's one-hour
//! mobile JWT (audience `dynwork-mobile`). Both are verified by the relay's
//! own functions against the instance's token authority, so the rules stay in
//! one place. The person always comes from the token, never from a body or a
//! path.
//!
//! A mobile JWT names the phone's device row in the relay, and that row must
//! still be live: revoking a phone refuses its next request at once.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use dynwork_relay::auth::{AuthError, bearer, verify_desktop};
use dynwork_relay::ids::PeerKind;
use dynwork_relay::mobile_auth::jwt;
use dynwork_relay::store::RelayStore;
use systemprompt::identifiers::UserId;

use crate::router::error::HttpFault;
use crate::router::state::BrainState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caller(pub UserId);

impl FromRequestParts<BrainState> for Caller {
    type Rejection = HttpFault;

    async fn from_request_parts(parts: &mut Parts, state: &BrainState) -> Result<Self, HttpFault> {
        let token = bearer(&parts.headers)?;
        let authority = state.authority.as_ref();
        let desktop = match verify_desktop(authority, token) {
            Ok(identity) => return Ok(Self(identity.user)),
            Err(refused) => refused,
        };
        let phone = jwt::verify(authority, token).inspect_err(|mobile| {
            tracing::debug!(%desktop, %mobile, "dynbrain: bearer refused");
        })?;
        let live = state
            .devices
            .list_devices(&phone.user)
            .await?
            .iter()
            .any(|row| {
                row.id == phone.device && row.kind == PeerKind::Phone && row.revoked_at.is_none()
            });
        if !live {
            tracing::debug!(device = %phone.device, "dynbrain: phone device is not live");
            return Err(AuthError::UnknownDevice.into());
        }
        Ok(Self(phone.user))
    }
}

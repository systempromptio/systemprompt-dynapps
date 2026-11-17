//! The desktop API's handlers and what they share: who the caller is, which
//! desktop peer it speaks for, the `If-Match` revision and the change hint.
//!
//! The caller presents the `DynWork` JWT exactly as the relay checks it
//! ([`dynwork_relay::auth::verify_desktop`]), and every row read or written is
//! scoped to that JWT's user. Bodies are parsed only after that check.

pub mod changes;
pub mod routines;
pub mod runs;
pub mod templates;

use axum::http::{HeaderMap, header};
use dynwork_relay::auth::{DesktopIdentity, bearer, verify_desktop};
use dynwork_relay::ids::PeerId;
use dynwork_relay::ws::PEER_HEADER;
use systemprompt::identifiers::UserId;

use crate::error::RoutinesError;
use crate::state::RoutinesState;

pub const HINT_KIND: &str = "routines";
pub const TEMPLATES_HINT_KIND: &str = "templates";

pub(crate) fn caller(
    state: &RoutinesState,
    headers: &HeaderMap,
) -> Result<DesktopIdentity, RoutinesError> {
    Ok(verify_desktop(state.authority.as_ref(), bearer(headers)?)?)
}

pub(crate) fn caller_peer(headers: &HeaderMap) -> Result<Option<PeerId>, RoutinesError> {
    let Some(value) = headers.get(PEER_HEADER) else {
        return Ok(None);
    };
    let text = value.to_str().map_err(|error| {
        tracing::debug!(%error, "x-dynwork-peer is not ASCII");
        RoutinesError::BadRequest("x-dynwork-peer is 32 hex digits")
    })?;
    PeerId::parse_hex(text).map(Some).map_err(|error| {
        tracing::debug!(%error, "x-dynwork-peer is not a peer id");
        RoutinesError::BadRequest("x-dynwork-peer is 32 hex digits")
    })
}

pub(crate) fn if_match(headers: &HeaderMap) -> Result<Option<i64>, RoutinesError> {
    let Some(value) = headers.get(header::IF_MATCH) else {
        return Ok(None);
    };
    value
        .to_str()
        .ok()
        .map(|text| text.trim().trim_matches('"'))
        .and_then(|text| text.parse::<i64>().ok())
        .filter(|revision| *revision > 0)
        .map(Some)
        .ok_or(RoutinesError::BadRequest("If-Match is a revision number"))
}

// Why: the hint is advisory; a failure is logged and the desktops still poll.
pub async fn hint(state_pool: &sqlx::PgPool, user: &UserId, except: Option<PeerId>) {
    if let Err(error) = dynwork_relay::hint::publish(state_pool, user, except, HINT_KIND).await {
        tracing::warn!(%error, "dynwork routines change hint not published");
    }
}

// Why: advisory, like the per-person hint: every desktop polls GET /templates
// anyway.
pub async fn templates_hint(pool: &sqlx::PgPool) {
    if let Err(error) = dynwork_relay::hint::publish_to_all(pool, TEMPLATES_HINT_KIND).await {
        tracing::warn!(%error, "dynwork templates change hint not published");
    }
}

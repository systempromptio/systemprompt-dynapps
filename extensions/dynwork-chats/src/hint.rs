//! The change hint: how the user's other desktops learn, without polling,
//! that a chat of theirs changed.
//!
//! Every write that gives a thread its next `change_seq` is followed, once the
//! store has committed it, by [`Hints::changed`]: a thread created, renamed,
//! archived or deleted, a reply that starts or ends, a thread the inactivity
//! job tombstones. The production [`RelayHints`] is the relay's own
//! `dynwork_relay::hint::publish` (a Postgres `NOTIFY` its listener turns into
//! `{"t":"chats.changed"}` for the person's desktops, except the caller's own).
//! The hint carries no data: a desktop that gets it calls `GET /changes`.
//!
//! A hint is advisory. A failed publish is logged and ignored, because every
//! client also polls the feed; the route's answer never depends on it.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use axum::http::HeaderMap;
use dynwork_relay::ids::PeerId;
use dynwork_relay::ws::PEER_HEADER;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub const KIND: &str = "chats";

/// Tells a user's desktops that their chats changed. `#[async_trait]` because
/// [`crate::state::Shared`] holds it as `Arc<dyn Hints>`: the relay's
/// `NOTIFY` in production, a recorder in the suites.
#[async_trait]
pub trait Hints: Send + Sync + Debug {
    async fn changed(&self, user: &UserId, except: Option<PeerId>);
}

/// [`Hints`] over the relay's `hint::publish`.
#[derive(Debug)]
pub struct RelayHints {
    pool: Arc<PgPool>,
}

impl RelayHints {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Hints for RelayHints {
    async fn changed(&self, user: &UserId, except: Option<PeerId>) {
        if let Err(error) = dynwork_relay::hint::publish(&self.pool, user, except, KIND).await {
            tracing::warn!(%error, %user, "dynwork chats change hint not published");
        }
    }
}

/// The desktop peer that made the request, from `X-Dynwork-Peer`: the one
/// desktop that does not need the hint. A header that is absent or not a peer
/// id means every desktop gets it.
pub fn caller_peer(headers: &HeaderMap) -> Option<PeerId> {
    let value = headers.get(PEER_HEADER)?;
    let text = value.to_str().ok()?;
    PeerId::parse_hex(text)
        .inspect_err(|error| tracing::debug!(%error, "x-dynwork-peer is not a peer id"))
        .ok()
}

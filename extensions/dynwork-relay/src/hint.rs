//! The change hint: how another extension tells a user's connected desktops
//! that something of theirs changed on the server.
//!
//! A writer calls [`publish`] after its transaction commits. That is a
//! Postgres `NOTIFY` on [`CHANNEL`], so the writer needs no handle on the
//! relay's process and the hint reaches whichever process holds the
//! desktops' sockets. The relay's listener ([`spawn`]) hands each
//! notification to [`Hub::notify_user_desktops`], which writes the control
//! frame `{"t":"<kind>.changed"}` to the user's desktops but the caller's.
//!
//! A hint is advisory. One sent while the listener reconnects, or to a
//! desktop that is offline or whose queue is full, is lost; the desktop
//! polls its feed anyway.

use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use systemprompt::identifiers::UserId;
use tokio_util::sync::CancellationToken;

use crate::hub::Hub;
use crate::ids::PeerId;
use crate::state::RelayState;

pub const CHANNEL: &str = "dynwork_desktop_hints";
const RETRY: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
struct Hint {
    user: UserId,
    except: Option<PeerId>,
    kind: String,
}

pub async fn publish(
    pool: &PgPool,
    user: &UserId,
    except: Option<PeerId>,
    kind: &str,
) -> Result<(), sqlx::Error> {
    // JSON: outgoing fixed-shape construction (the NOTIFY payload).
    let payload = json!({ "user": user.as_str(), "except": except, "kind": kind }).to_string();
    sqlx::query_scalar!(
        r#"SELECT 1 AS "sent!" FROM pg_notify($1, $2)"#,
        CHANNEL,
        payload
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}

pub fn spawn(state: &RelayState, pool: Arc<PgPool>) {
    let hub = Arc::clone(&state.hub);
    state
        .tasks
        .spawn_cancellable("dynwork_relay_hints", move |token| listen(hub, pool, token));
}

pub async fn listen(hub: Arc<Hub>, pool: Arc<PgPool>, token: CancellationToken) {
    let Some(mut listener) = subscribe(&pool, &token).await else {
        return;
    };
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            received = listener.recv() => match received {
                Ok(notification) => {
                    deliver(&hub, notification.payload());
                },
                Err(error) => {
                    tracing::warn!(%error, "dynwork desktop hint listener lost its connection");
                    if !pause(&token).await {
                        return;
                    }
                },
            },
        }
    }
}

pub fn deliver(hub: &Hub, payload: &str) -> usize {
    match serde_json::from_str::<Hint>(payload) {
        Ok(hint) => hub.notify_user_desktops(&hint.user, hint.except, &hint.kind),
        Err(error) => {
            tracing::warn!(%error, "dynwork desktop hint payload is not a hint");
            0
        },
    }
}

async fn subscribe(pool: &PgPool, token: &CancellationToken) -> Option<PgListener> {
    loop {
        match connect(pool).await {
            Ok(listener) => return Some(listener),
            Err(error) => {
                tracing::warn!(%error, "dynwork desktop hint listener could not subscribe");
                if !pause(token).await {
                    return None;
                }
            },
        }
    }
}

async fn connect(pool: &PgPool) -> Result<PgListener, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(CHANNEL).await?;
    Ok(listener)
}

async fn pause(token: &CancellationToken) -> bool {
    tokio::select! {
        () = token.cancelled() => false,
        () = tokio::time::sleep(RETRY) => true,
    }
}

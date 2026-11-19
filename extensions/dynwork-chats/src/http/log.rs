//! `GET /threads/{id}/log?after=<seq>&follow=1`: a thread's events as
//! server-sent events.
//!
//! It replays the stored events after `after` (or after `Last-Event-ID`,
//! whichever is later) and, with `follow=1`, goes on with the live ones until
//! the client goes away, the token expires, or, for a phone, a heartbeat finds
//! its device no longer live (revoked); `follow=0` (the default) closes after
//! the replay.
//! Each event is one `data:` line holding the `OpenCode` v2 envelope, with
//! `id:` its `seq`, so a reader resumes exactly where it stopped; a comment
//! heartbeat goes out every [`HEARTBEAT`] (the state's
//! [`Limits`](crate::state::Limits)). The live feed is subscribed before the
//! store is read and anything at or below the last `seq` sent is dropped, so
//! a resume has no gap and no duplicate; a reader that falls behind the feed
//! reads the store again. A follower going away never stops the reply.

use std::collections::VecDeque;
use std::convert::Infallible;
use std::time::Duration;

use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use dynwork_relay::ids::DeviceId;
use futures_util::StreamExt;
use serde::Deserialize;
use systemprompt::identifiers::UserId;
use tokio::sync::broadcast::error::RecvError;

use crate::auth::{Caller, Client};
use crate::clock::{Clock, SystemClock};
use crate::error::ChatsError;
use crate::http::response::no_store;
use crate::ids::ThreadId;
use crate::reply::events::envelope;
use crate::reply::registry::Follow;
use crate::reply::start::settle_if_idle;
use crate::state::ChatsState;
use crate::store::EventRecord;

pub const REPLAY_PAGE: u32 = 200;
pub const HEARTBEAT: Duration = Duration::from_secs(15);
pub const LAST_EVENT_ID: &str = "last-event-id";

/// The query of a log request.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogQuery {
    pub after: Option<i64>,
    pub follow: Option<String>,
}

pub fn start_after(query: &LogQuery, headers: &HeaderMap) -> Result<i64, ChatsError> {
    let header = headers
        .get(LAST_EVENT_ID)
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|text| text.trim().parse::<i64>().ok())
                .ok_or(ChatsError::BadRequest("Last-Event-ID must be a seq"))
        })
        .transpose()?;
    let after = query.after.unwrap_or(0).max(header.unwrap_or(0));
    if query.after.is_some_and(|after| after < 0) || header.is_some_and(|seq| seq < 0) {
        return Err(ChatsError::BadRequest("a seq is never negative"));
    }
    Ok(after)
}

pub fn wants_follow(query: &LogQuery) -> Result<bool, ChatsError> {
    match query.follow.as_deref() {
        None | Some("0" | "false") => Ok(false),
        Some("1" | "true") => Ok(true),
        Some(_) => Err(ChatsError::BadRequest("follow is 0 or 1")),
    }
}

pub async fn log(
    caller: Caller,
    State(state): State<ChatsState>,
    path: Result<Path<String>, PathRejection>,
    query: Result<Query<LogQuery>, QueryRejection>,
    headers: HeaderMap,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let thread: ThreadId = id.parse()?;
    let Query(query) = query?;
    let after = start_after(&query, &headers)?;
    let follow = wants_follow(&query)?;
    state
        .store
        .get_thread(&caller.user, &thread)
        .await?
        .filter(|record| record.deleted_at.is_none())
        .ok_or(ChatsError::NotFound)?;
    settle_if_idle(&state, &caller.user, &thread).await?;
    let live = follow.then(|| state.replies.follow(&thread));
    let heartbeat = state.limits.heartbeat;
    let ends = lifetime(state.clone(), caller.clone());
    let reader = Reader {
        state,
        user: caller.user,
        thread,
        last: after,
        backlog: VecDeque::new(),
        replayed: false,
        live,
    };
    let events = futures_util::stream::unfold(reader, Reader::next).take_until(ends);
    let mut response = Sse::new(events)
        .keep_alive(KeepAlive::new().interval(heartbeat))
        .into_response();
    response
        .headers_mut()
        .insert("x-accel-buffering", HeaderValue::from_static("no"));
    Ok(no_store(response))
}

// Why: core checks a token's expiry against wall-clock time, so the stream's
// end reads the system clock, not the state's injected one.
async fn lifetime(state: ChatsState, caller: Caller) {
    let left = time_left(caller.expires_at, SystemClock.now());
    let revoked = async {
        match caller.client {
            Client::Phone(device) => phone_revoked(&state, &caller.user, device).await,
            Client::Desktop => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        () = tokio::time::sleep(left) => {
            tracing::debug!(user = %caller.user, "dynwork chats log closed: the token expired");
        },
        () = revoked => {},
    }
}

fn time_left(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> Duration {
    (expires_at - now).to_std().unwrap_or(Duration::ZERO)
}

async fn phone_revoked(state: &ChatsState, user: &UserId, device: DeviceId) {
    let every = state.limits.heartbeat;
    let mut beats = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
    loop {
        beats.tick().await;
        match state.devices.phone_is_live(user, device).await {
            Ok(true) => {},
            Ok(false) => {
                tracing::debug!(%device, "dynwork chats log closed: the phone is not live");
                return;
            },
            Err(error) => {
                tracing::warn!(error = ?error, %device, "dynwork chats log closed: the phone could not be checked");
                return;
            },
        }
    }
}

pub fn sse_event(record: &EventRecord) -> Event {
    Event::default()
        .id(record.seq.to_string())
        .data(envelope(record).to_string())
}

#[derive(Debug)]
struct Reader {
    state: ChatsState,
    user: UserId,
    thread: ThreadId,
    last: i64,
    backlog: VecDeque<EventRecord>,
    replayed: bool,
    live: Option<Follow>,
}

impl Reader {
    async fn next(mut self) -> Option<(Result<Event, Infallible>, Self)> {
        loop {
            if let Some(record) = self.backlog.pop_front() {
                if record.seq <= self.last {
                    continue;
                }
                self.last = record.seq;
                let event = sse_event(&record);
                return Some((Ok(event), self));
            }
            if !self.replayed {
                let page = self.page().await?;
                self.replayed = page.len() < REPLAY_PAGE as usize;
                self.backlog.extend(page);
                continue;
            }
            let live = self.live.as_mut()?;
            match live.receiver.recv().await {
                Ok(record) if record.seq <= self.last => {},
                Ok(record) if record.seq == self.last + 1 => self.backlog.push_back(record),
                Ok(_) | Err(RecvError::Lagged(_)) => self.replayed = false,
                Err(RecvError::Closed) => return None,
            }
        }
    }

    async fn page(&self) -> Option<Vec<EventRecord>> {
        match self
            .state
            .store
            .events_after(&self.user, &self.thread, self.last, REPLAY_PAGE)
            .await
        {
            Ok(page) => page,
            Err(error) => {
                tracing::error!(error = ?error, thread = %self.thread, "dynwork chats log could not read events");
                None
            },
        }
    }
}

//! One tunnel's loop: forward envelopes both ways, keep the heartbeat,
//! enforce the limits and close with the right code.

use std::future::poll_fn;
use std::task::Poll;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use futures_util::StreamExt;
use tokio::sync::OwnedSemaphorePermit;
use tokio_util::sync::CancellationToken;

use crate::audit::{AuditEvent, ConnectionStats};
use crate::envelope::{self, EnvelopeError};
use crate::hub::rate::RateLimiter;
use crate::hub::{Attachment, CloseReason, Endpoint};
use crate::state::RelayState;

// Why: RFC 6455 section 7.1.1: the server waits for the peer's close frame
// before dropping TCP, so a peer still flushing pongs reads the close code.
const CLOSE_HANDSHAKE: Duration = Duration::from_secs(2);

pub(super) struct Tunnel {
    pub(super) state: RelayState,
    pub(super) endpoint: Endpoint,
    pub(super) lifetime: Duration,
    pub(super) permit: OwnedSemaphorePermit,
}

enum End {
    Close(CloseReason),
    PeerClosed,
    Transport,
}

impl End {
    const fn label(&self) -> &'static str {
        match self {
            Self::Close(reason) => reason.text(),
            Self::PeerClosed => "peer closed",
            Self::Transport => "transport error",
        }
    }
}

#[derive(Default)]
struct Counters {
    envelopes_in: u64,
    envelopes_out: u64,
}

async fn send_within(socket: &mut WebSocket, message: Message, limit: Duration) -> bool {
    match tokio::time::timeout(limit, socket.send(message)).await {
        Ok(Ok(())) => true,
        Ok(Err(error)) => {
            tracing::debug!(error = %error, "dynwork relay socket write failed");
            false
        },
        Err(_elapsed) => {
            tracing::debug!("dynwork relay socket write timed out");
            false
        },
    }
}

async fn send_close(socket: &mut WebSocket, reason: CloseReason) {
    let frame = CloseFrame {
        code: reason.code(),
        reason: Utf8Bytes::from_static(reason.text()),
    };
    if !send_within(socket, Message::Close(Some(frame)), CLOSE_HANDSHAKE).await {
        return;
    }
    let answered = tokio::time::timeout(CLOSE_HANDSHAKE, async {
        while let Some(Ok(message)) = socket.next().await {
            if matches!(message, Message::Close(_)) {
                break;
            }
        }
    })
    .await;
    if answered.is_err() {
        tracing::debug!("dynwork relay peer did not answer the close frame");
    }
}

pub(super) async fn run(mut socket: WebSocket, tunnel: Tunnel, shutdown: CancellationToken) {
    let Tunnel {
        state,
        endpoint,
        lifetime,
        permit,
    } = tunnel;
    let started = Instant::now();
    let Some(mut attachment) = state.hub.attach(&endpoint, started) else {
        send_close(&mut socket, CloseReason::Revoked).await;
        return;
    };
    state.audit.record(&AuditEvent::Connect {
        user: endpoint.user.clone(),
        device: endpoint.device,
        kind: endpoint.kind,
    });
    let mut counters = Counters::default();
    let end = drive(
        &mut socket,
        &state,
        &endpoint,
        &mut attachment,
        lifetime,
        &shutdown,
        &mut counters,
    )
    .await;
    state
        .hub
        .detach(&endpoint, attachment.conn, &mut attachment.rx);
    if let End::Close(reason) = end {
        send_close(&mut socket, reason).await;
    }
    if let Err(error) = state
        .store
        .touch_device(endpoint.device, state.clock.now())
        .await
    {
        tracing::warn!(error = %error, device_id = %endpoint.device, "dynwork relay last-seen not recorded");
    }
    state.audit.record(&AuditEvent::Disconnect {
        user: endpoint.user.clone(),
        device: endpoint.device,
        kind: endpoint.kind,
        reason: end.label(),
        stats: ConnectionStats {
            envelopes_in: counters.envelopes_in,
            envelopes_out: counters.envelopes_out,
            duration_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        },
    });
    drop(permit);
}

#[expect(
    clippy::too_many_arguments,
    reason = "one loop owns the socket, the attachment and its counters; splitting it would share them through a struct of borrows"
)]
async fn drive(
    socket: &mut WebSocket,
    state: &RelayState,
    endpoint: &Endpoint,
    attachment: &mut Attachment,
    lifetime: Duration,
    shutdown: &CancellationToken,
    counters: &mut Counters,
) -> End {
    let limits = &state.limits;
    let deadline = tokio::time::sleep(lifetime);
    tokio::pin!(deadline);
    let start = tokio::time::Instant::now();
    let mut heartbeat =
        tokio::time::interval_at(start + limits.heartbeat_interval, limits.heartbeat_interval);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut last_heard = Instant::now();
    let mut limiter = RateLimiter::new(&limits.rate, last_heard);
    let mut drain = state.tasks.stream_drain().watch();
    let control = std::sync::Arc::clone(&attachment.control);
    loop {
        tokio::select! {
            biased;
            () = shutdown.cancelled() => return End::Close(CloseReason::Drain),
            () = poll_fn(|cx| if drain.poll_ended(cx) { Poll::Ready(()) } else { Poll::Pending }) => {
                return End::Close(CloseReason::Drain);
            },
            reason = control.closed() => return End::Close(reason),
            () = &mut deadline => return End::Close(CloseReason::Reconnect),
            _ = heartbeat.tick() => {
                if last_heard.elapsed() > limits.heartbeat_timeout {
                    return End::Close(CloseReason::Heartbeat);
                }
                if !send_within(socket, Message::Ping(Bytes::new()), limits.heartbeat_timeout).await {
                    return End::Transport;
                }
            },
            outbound = attachment.rx.recv() => {
                let Some(envelope) = outbound else {
                    return End::Close(control.reason().unwrap_or(CloseReason::Reconnect));
                };
                if !send_within(socket, Message::Binary(envelope), limits.heartbeat_timeout).await {
                    return End::Transport;
                }
                counters.envelopes_out += 1;
            },
            incoming = socket.next() => {
                last_heard = Instant::now();
                match incoming {
                    Some(Ok(Message::Binary(bytes))) => {
                        if let Err(reason) = forward(state, endpoint, &mut limiter, bytes) {
                            return End::Close(reason);
                        }
                        counters.envelopes_in += 1;
                    },
                    Some(Ok(Message::Text(_))) => return End::Close(CloseReason::BinaryOnly),
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => {},
                    Some(Ok(Message::Close(_))) | None => return End::PeerClosed,
                    Some(Err(error)) => {
                        tracing::debug!(error = %error, "dynwork relay socket read failed");
                        return End::Transport;
                    },
                }
            },
        }
    }
}

fn forward(
    state: &RelayState,
    endpoint: &Endpoint,
    limiter: &mut RateLimiter,
    bytes: Bytes,
) -> Result<(), CloseReason> {
    let now = Instant::now();
    if bytes.len() > state.limits.max_envelope {
        return Err(CloseReason::TooLarge);
    }
    if !limiter.admit(bytes.len(), now) {
        return Err(CloseReason::RateLimited);
    }
    let to = match envelope::recipient(&bytes, state.limits.max_envelope) {
        Ok(to) => to,
        Err(EnvelopeError::TooLarge { .. }) => return Err(CloseReason::TooLarge),
        Err(EnvelopeError::TooShort { .. }) => return Err(CloseReason::Malformed),
        Err(EnvelopeError::UnknownVersion(_)) => return Ok(()),
    };
    let mut owned: Vec<u8> = bytes.into();
    envelope::stamp_sender(&mut owned, endpoint.peer);
    let routed = state.hub.route(endpoint, to, Bytes::from(owned), now);
    tracing::trace!(outcome = ?routed, "dynwork relay envelope routed");
    Ok(())
}

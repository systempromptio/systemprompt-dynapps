//! In-process routing between the live tunnels of each user.
//!
//! Everything is keyed by `(user, peer id)`, so an envelope can only reach a
//! socket of the sender's own user, and only the other kind of end: a phone
//! talks to desktops, a desktop to phones. An envelope for a known peer that
//! is not connected is held in the ring buffer of the desktop end of the
//! pair and delivered, in order, before anything live when that peer
//! attaches. Routing never awaits under the lock, and one sender's envelopes
//! go through one bounded queue per recipient, so order per `(from, to)`
//! pair holds.
//!
//! The hub also tells a user's desktops when one of the user's phones
//! attaches or goes away, with a relay presence message
//! ([`crate::envelope::presence`]) on the same queue as the envelopes, so a
//! desktop sees a phone's last envelope before its departure.

pub mod control;
mod inner;
pub mod rate;
pub mod ring;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use axum::body::Bytes;
use systemprompt::identifiers::UserId;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{self};

pub use control::{CloseReason, Control};
pub use ring::Ring;

use inner::{Inner, Key, Known, Slot};

use crate::config::RingLimits;
use crate::envelope::Presence;
use crate::ids::{DeviceId, PeerId, PeerKind};

pub type ConnId = u64;

/// One authenticated end of a tunnel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub user: UserId,
    pub peer: PeerId,
    pub device: DeviceId,
    pub kind: PeerKind,
}

/// A tunnel's receiving side, handed to its socket loop.
#[derive(Debug)]
pub struct Attachment {
    pub conn: ConnId,
    pub rx: mpsc::Receiver<Bytes>,
    pub control: Arc<Control>,
    pub flushed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Routed {
    Delivered,
    Held,
    Dropped(DropReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    ToSelf,
    UnknownPeer,
    SameKind,
    NotHeld,
}

#[derive(Debug)]
pub struct Hub {
    inner: Mutex<Inner>,
    ring: RingLimits,
    queue: usize,
    next: AtomicU64,
}

impl Hub {
    pub fn new(ring: RingLimits, queue: usize) -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            ring,
            queue: queue.max(1),
            next: AtomicU64::new(1),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn attach(&self, endpoint: &Endpoint, now: Instant) -> Option<Attachment> {
        let control = Arc::new(Control::default());
        let conn = self.next.fetch_add(1, Ordering::Relaxed);
        let key = Key::of(&endpoint.user, endpoint.peer);
        let mut inner = self.lock();
        if inner.revoked.contains(&endpoint.device) {
            return None;
        }
        let mut held = Vec::new();
        for (owner, ring) in &mut inner.rings {
            if owner.user == endpoint.user {
                held.extend(ring.take_for(endpoint.peer, now));
            }
        }
        let (tx, rx) = mpsc::channel(self.queue + held.len());
        let flushed = held.len();
        for envelope in held {
            if tx.try_send(envelope).is_err() {
                break;
            }
        }
        inner.known.insert(
            key.clone(),
            Known {
                device: endpoint.device,
                kind: endpoint.kind,
            },
        );
        let replaced = inner.live.insert(
            key,
            Slot {
                conn,
                device: endpoint.device,
                kind: endpoint.kind,
                tx,
                control: Arc::clone(&control),
            },
        );
        match endpoint.kind {
            PeerKind::Phone => {
                inner.tell_desktops(&endpoint.user, endpoint.peer, Presence::Connected);
            },
            PeerKind::Desktop => {
                inner.tell_desktop_who_is_on(&Key::of(&endpoint.user, endpoint.peer));
            },
        }
        drop(inner);
        if let Some(old) = replaced {
            old.control.close(CloseReason::Replaced);
        }
        Some(Attachment {
            conn,
            rx,
            control,
            flushed,
        })
    }

    pub fn detach(
        &self,
        endpoint: &Endpoint,
        conn: ConnId,
        rx: &mut mpsc::Receiver<Bytes>,
    ) -> usize {
        let key = Key::of(&endpoint.user, endpoint.peer);
        let mut inner = self.lock();
        match inner.live.get(&key).map(|slot| slot.conn == conn) {
            Some(true) => {
                inner.remove(&key);
            },
            Some(false) => return 0,
            None => {},
        }
        if inner.revoked.contains(&endpoint.device) {
            return 0;
        }
        inner.recover(endpoint, rx, self.ring, Instant::now())
    }

    pub fn route(&self, from: &Endpoint, to: PeerId, envelope: Bytes, now: Instant) -> Routed {
        if to == from.peer {
            return Routed::Dropped(DropReason::ToSelf);
        }
        let key = Key::of(&from.user, to);
        let mut inner = self.lock();
        let Some(known) = inner.known.get(&key).copied() else {
            return Routed::Dropped(DropReason::UnknownPeer);
        };
        if known.kind == from.kind {
            return Routed::Dropped(DropReason::SameKind);
        }
        let sent = match inner.live.get(&key) {
            Some(slot) => slot.tx.try_send(envelope),
            None => Err(TrySendError::Closed(envelope)),
        };
        let envelope = match sent {
            Ok(()) => return Routed::Delivered,
            Err(TrySendError::Full(envelope)) => {
                if let Some(slot) = inner.remove(&key) {
                    slot.control.close(CloseReason::SlowConsumer);
                }
                envelope
            },
            Err(TrySendError::Closed(envelope)) => {
                inner.remove(&key);
                envelope
            },
        };
        let owner = if from.kind == PeerKind::Desktop {
            Key::of(&from.user, from.peer)
        } else {
            key
        };
        let ring = self.ring;
        let held = inner
            .rings
            .entry(owner)
            .or_insert_with(|| Ring::new(ring))
            .push(to, envelope, now);
        drop(inner);
        if held {
            Routed::Held
        } else {
            Routed::Dropped(DropReason::NotHeld)
        }
    }

    pub fn revoke(&self, device: DeviceId) -> usize {
        self.lock().revoke(device, Instant::now())
    }

    pub fn sweep_rings(&self, now: Instant) {
        self.lock().rings.retain(|_, ring| {
            ring.expire(now);
            !ring.is_empty()
        });
    }

    pub fn rings(&self) -> usize {
        self.lock().rings.len()
    }

    pub fn is_online(&self, device: DeviceId) -> bool {
        self.lock().live.values().any(|slot| slot.device == device)
    }

    pub fn tunnels(&self) -> usize {
        self.lock().live.len()
    }

    pub fn held(&self, user: &UserId, desktop: PeerId) -> usize {
        self.lock()
            .rings
            .get(&Key::of(user, desktop))
            .map_or(0, Ring::len)
    }
}

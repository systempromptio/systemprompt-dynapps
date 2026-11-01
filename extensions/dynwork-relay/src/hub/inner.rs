//! The hub's state under its lock: live slots, known peers, the desktops'
//! rings and the revoked devices, and the presence messages a change in the
//! live set sends.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use axum::body::Bytes;
use systemprompt::identifiers::UserId;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

use super::{CloseReason, ConnId, Control, Endpoint, Ring};
use crate::config::RingLimits;
use crate::envelope::{FROM_AT, Presence, VERSION, presence};
use crate::ids::{DeviceId, PEER_ID_LEN, PeerId, PeerKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Key {
    pub(super) user: UserId,
    pub(super) peer: PeerId,
}

impl Key {
    pub(super) fn of(user: &UserId, peer: PeerId) -> Self {
        Self {
            user: user.clone(),
            peer,
        }
    }
}

#[derive(Debug)]
pub(super) struct Slot {
    pub(super) conn: ConnId,
    pub(super) device: DeviceId,
    pub(super) kind: PeerKind,
    pub(super) tx: mpsc::Sender<Bytes>,
    pub(super) control: Arc<Control>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Known {
    pub(super) device: DeviceId,
    pub(super) kind: PeerKind,
}

#[derive(Debug, Default)]
pub(super) struct Inner {
    pub(super) live: HashMap<Key, Slot>,
    pub(super) known: HashMap<Key, Known>,
    pub(super) rings: HashMap<Key, Ring>,
    pub(super) revoked: HashSet<DeviceId>,
}

impl Inner {
    // Why: envelopes still queued when a tunnel closes never reached its
    // socket; the ring of the desktop end keeps them, ahead of anything held
    // since, for the reconnect. The relay's presence messages are dropped.
    pub(super) fn recover(
        &mut self,
        endpoint: &Endpoint,
        rx: &mut mpsc::Receiver<Bytes>,
        limits: RingLimits,
        now: Instant,
    ) -> usize {
        rx.close();
        let mut by_owner: HashMap<PeerId, Vec<Bytes>> = HashMap::new();
        while let Ok(envelope) = rx.try_recv() {
            if envelope.first() != Some(&VERSION) {
                continue;
            }
            let owner = match endpoint.kind {
                PeerKind::Desktop => Some(endpoint.peer),
                PeerKind::Phone => envelope
                    .get(FROM_AT..FROM_AT + PEER_ID_LEN)
                    .and_then(PeerId::from_slice),
            };
            if let Some(owner) = owner {
                by_owner.entry(owner).or_default().push(envelope);
            }
        }
        let mut held = 0;
        for (owner, envelopes) in by_owner {
            held += self
                .rings
                .entry(Key::of(&endpoint.user, owner))
                .or_insert_with(|| Ring::new(limits))
                .restore(endpoint.peer, envelopes, now);
        }
        held
    }

    pub(super) fn revoke(&mut self, device: DeviceId, now: Instant) -> usize {
        self.revoked.insert(device);
        let mut keys: Vec<Key> = self
            .known
            .iter()
            .filter(|(_, known)| known.device == device)
            .map(|(key, _)| key.clone())
            .collect();
        let live: Vec<Key> = self
            .live
            .iter()
            .filter(|(_, slot)| slot.device == device)
            .map(|(key, _)| key.clone())
            .collect();
        for key in live {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        self.known.retain(|_, known| known.device != device);
        let mut closed = 0;
        for key in keys {
            self.rings.remove(&key);
            for (owner, ring) in &mut self.rings {
                if owner.user == key.user {
                    ring.remove_for(key.peer, now);
                }
            }
            if let Some(slot) = self.remove(&key) {
                slot.control.close(CloseReason::Revoked);
                closed += 1;
            }
        }
        closed
    }

    pub(super) fn remove(&mut self, key: &Key) -> Option<Slot> {
        let slot = self.live.remove(key)?;
        if slot.kind == PeerKind::Phone {
            self.tell_desktops(&key.user, key.peer, Presence::Disconnected);
        }
        Some(slot)
    }

    pub(super) fn tell_desktops(&mut self, user: &UserId, phone: PeerId, event: Presence) {
        let mut slow = Vec::new();
        for (key, slot) in &self.live {
            if &key.user != user || slot.kind != PeerKind::Desktop {
                continue;
            }
            let message = Bytes::from(presence(key.peer, phone, event));
            if let Err(TrySendError::Full(_)) = slot.tx.try_send(message) {
                slow.push(key.clone());
            }
        }
        for key in slow {
            if let Some(slot) = self.live.remove(&key) {
                slot.control.close(CloseReason::SlowConsumer);
            }
        }
    }

    pub(super) fn tell_desktop_who_is_on(&self, desktop: &Key) {
        let Some(slot) = self.live.get(desktop) else {
            return;
        };
        for (key, phone) in &self.live {
            if key.user == desktop.user && phone.kind == PeerKind::Phone {
                let message = Bytes::from(presence(desktop.peer, key.peer, Presence::Connected));
                if slot.tx.try_send(message).is_err() {
                    break;
                }
            }
        }
    }
}

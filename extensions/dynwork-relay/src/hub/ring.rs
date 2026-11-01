//! A desktop's ring buffer of encrypted envelopes held for a peer that is
//! not connected. Bounded by count, bytes and age; the oldest go first.

use std::collections::VecDeque;
use std::time::Instant;

use axum::body::Bytes;

use crate::config::RingLimits;
use crate::ids::PeerId;

#[derive(Debug, Clone)]
struct Held {
    to: PeerId,
    envelope: Bytes,
    at: Instant,
}

#[derive(Debug, Clone)]
pub struct Ring {
    held: VecDeque<Held>,
    bytes: usize,
    limits: RingLimits,
    evicted: u64,
}

impl Ring {
    pub const fn new(limits: RingLimits) -> Self {
        Self {
            held: VecDeque::new(),
            bytes: 0,
            limits,
            evicted: 0,
        }
    }

    pub fn push(&mut self, to: PeerId, envelope: Bytes, now: Instant) -> bool {
        self.expire(now);
        if envelope.len() > self.limits.bytes || self.limits.envelopes == 0 {
            return false;
        }
        self.bytes += envelope.len();
        self.held.push_back(Held {
            to,
            envelope,
            at: now,
        });
        while self.held.len() > self.limits.envelopes || self.bytes > self.limits.bytes {
            self.evict_oldest();
        }
        true
    }

    pub fn take_for(&mut self, to: PeerId, now: Instant) -> Vec<Bytes> {
        self.expire(now);
        let mut taken = Vec::new();
        let mut kept = VecDeque::with_capacity(self.held.len());
        for held in self.held.drain(..) {
            if held.to == to {
                self.bytes -= held.envelope.len();
                taken.push(held.envelope);
            } else {
                kept.push_back(held);
            }
        }
        self.held = kept;
        taken
    }

    pub fn restore(&mut self, to: PeerId, envelopes: Vec<Bytes>, now: Instant) -> usize {
        let mut restored = 0;
        for envelope in envelopes.into_iter().rev() {
            if envelope.len() > self.limits.bytes {
                continue;
            }
            self.bytes += envelope.len();
            self.held.push_front(Held {
                to,
                envelope,
                at: now,
            });
            restored += 1;
        }
        while self.held.len() > self.limits.envelopes || self.bytes > self.limits.bytes {
            self.evict_oldest();
        }
        restored
    }

    pub fn remove_for(&mut self, to: PeerId, now: Instant) {
        self.expire(now);
        let before = self.held.len();
        self.held.retain(|held| held.to != to);
        if self.held.len() != before {
            self.bytes = self.held.iter().map(|held| held.envelope.len()).sum();
        }
    }

    pub fn len(&self) -> usize {
        self.held.len()
    }

    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    pub const fn bytes(&self) -> usize {
        self.bytes
    }

    pub const fn evicted(&self) -> u64 {
        self.evicted
    }

    pub fn expire(&mut self, now: Instant) {
        while self
            .held
            .front()
            .is_some_and(|held| now.saturating_duration_since(held.at) > self.limits.ttl)
        {
            self.evict_oldest();
        }
    }

    fn evict_oldest(&mut self) {
        if let Some(held) = self.held.pop_front() {
            self.bytes -= held.envelope.len();
            self.evicted += 1;
        }
    }
}

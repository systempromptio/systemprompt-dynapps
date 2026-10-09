//! Short-lived in-process caches for the authorization reads every gateway
//! request repeats.
//!
//! The route entitlement guard and the subject attribute providers run on
//! every `/v1/messages` call, and without a cache each one is a database
//! round trip for data that changes a few times a day. Entries live for
//! [`AUTHZ_TTL`]. One process-wide generation counter backs
//! [`invalidate_all`]: an entry stamped with an older generation is a miss,
//! so an access-control sync apply takes effect at once on the replica that
//! ran it and within [`AUTHZ_TTL`] on every other replica.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

pub const AUTHZ_TTL: Duration = Duration::from_secs(30);

const MAX_ENTRIES: usize = 10_000;

static GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn invalidate_all() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

#[derive(Debug)]
struct Entry<V> {
    value: V,
    stored_at: Instant,
    generation: u64,
}

impl<V> Entry<V> {
    fn is_fresh(&self, ttl: Duration, now: Instant, current: u64) -> bool {
        self.generation == current && now.duration_since(self.stored_at) < ttl
    }
}

#[derive(Debug)]
pub struct TtlCache<K, V> {
    ttl: Duration,
    entries: Mutex<HashMap<K, Entry<V>>>,
}

impl<K: Eq + Hash, V: Clone> TtlCache<K, V> {
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<K, Entry<V>>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[must_use]
    pub fn get(&self, key: &K) -> Option<V> {
        let current = generation();
        let now = Instant::now();
        self.lock()
            .get(key)
            .filter(|entry| entry.is_fresh(self.ttl, now, current))
            .map(|entry| entry.value.clone())
    }

    pub fn insert(&self, key: K, value: V) {
        let current = generation();
        let now = Instant::now();
        let mut entries = self.lock();
        if entries.len() >= MAX_ENTRIES {
            entries.retain(|_, entry| entry.is_fresh(self.ttl, now, current));
            if entries.len() >= MAX_ENTRIES {
                entries.clear();
            }
        }
        entries.insert(
            key,
            Entry {
                value,
                stored_at: now,
                generation: current,
            },
        );
    }
}

//! Per-connection rate limit: a token bucket for envelopes and one for bytes.

use std::time::Instant;

use crate::config::RateLimit;

#[derive(Debug, Clone, Copy)]
struct Bucket {
    capacity: f64,
    tokens: f64,
    refill_per_second: f64,
}

impl Bucket {
    const fn new(capacity: f64, refill_per_second: f64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_per_second,
        }
    }

    const fn refill(&mut self, elapsed_seconds: f64) {
        self.tokens = elapsed_seconds
            .mul_add(self.refill_per_second, self.tokens)
            .min(self.capacity);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RateLimiter {
    envelopes: Bucket,
    bytes: Bucket,
    last: Instant,
}

impl RateLimiter {
    pub const fn new(limit: &RateLimit, now: Instant) -> Self {
        Self {
            envelopes: Bucket::new(limit.message_burst, limit.messages_per_second),
            bytes: Bucket::new(limit.byte_burst, limit.bytes_per_second),
            last: now,
        }
    }

    pub fn admit(&mut self, len: usize, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last).as_secs_f64();
        self.last = now;
        self.envelopes.refill(elapsed);
        self.bytes.refill(elapsed);
        let size = len as f64;
        if self.envelopes.tokens < 1.0 || self.bytes.tokens < size {
            return false;
        }
        self.envelopes.tokens -= 1.0;
        self.bytes.tokens -= size;
        true
    }
}

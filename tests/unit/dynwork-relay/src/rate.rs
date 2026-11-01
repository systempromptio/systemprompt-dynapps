use std::time::{Duration, Instant};

use dynwork_relay::config::RateLimit;
use dynwork_relay::hub::rate::RateLimiter;

const LIMIT: RateLimit = RateLimit {
    messages_per_second: 10.0,
    message_burst: 5.0,
    bytes_per_second: 1_000.0,
    byte_burst: 2_000.0,
};

#[test]
fn the_burst_is_admitted_and_the_next_envelope_is_not() {
    let start = Instant::now();
    let mut limiter = RateLimiter::new(&LIMIT, start);
    for _ in 0..5 {
        assert!(limiter.admit(10, start));
    }
    assert!(!limiter.admit(10, start));
}

#[test]
fn the_budget_refills_with_time() {
    let start = Instant::now();
    let mut limiter = RateLimiter::new(&LIMIT, start);
    for _ in 0..5 {
        assert!(limiter.admit(10, start));
    }
    assert!(!limiter.admit(10, start));
    assert!(limiter.admit(10, start + Duration::from_millis(150)));
}

#[test]
fn the_byte_budget_refuses_large_envelopes_past_the_burst() {
    let start = Instant::now();
    let mut limiter = RateLimiter::new(&LIMIT, start);
    assert!(limiter.admit(1_500, start));
    assert!(!limiter.admit(1_000, start));
    assert!(limiter.admit(400, start));
    assert!(limiter.admit(1_000, start + Duration::from_secs(1)));
}

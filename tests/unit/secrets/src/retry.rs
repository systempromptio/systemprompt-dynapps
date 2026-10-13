//! Connect retry classification and the backoff schedule.

use std::time::Duration;

use systemprompt_secrets_onepassword::retry::{
    BASE_DELAY, MAX_DELAY, RetryPolicy, StatusClass, classify_status,
};

#[test]
fn success_codes() {
    assert_eq!(classify_status(200), StatusClass::Success);
    assert_eq!(classify_status(204), StatusClass::Success);
}

#[test]
fn throttle_and_server_errors_retry() {
    for code in [429, 500, 502, 503, 504] {
        assert_eq!(classify_status(code), StatusClass::Retryable, "{code}");
    }
}

#[test]
fn auth_and_missing_never_retry() {
    for code in [400, 401, 403, 404, 301, 302] {
        assert_eq!(classify_status(code), StatusClass::Fatal, "{code}");
    }
}

#[test]
fn delays_count_matches_retries() {
    assert!(RetryPolicy::new(0).delays().is_empty());
    assert_eq!(RetryPolicy::new(3).delays().len(), 3);
}

#[test]
fn delays_double_from_base() {
    let delays = RetryPolicy::new(3).delays();
    assert_eq!(delays, vec![BASE_DELAY, BASE_DELAY * 2, BASE_DELAY * 4]);
}

#[test]
fn delays_are_capped() {
    let delays = RetryPolicy::new(10).delays();
    assert!(delays.iter().all(|d| *d <= MAX_DELAY));
    assert_eq!(*delays.last().unwrap(), MAX_DELAY);
    assert!(delays.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn huge_attempt_does_not_overflow() {
    let policy = RetryPolicy {
        retries: 40,
        base: Duration::from_secs(1),
        max: Duration::from_secs(30),
    };
    assert_eq!(*policy.delays().last().unwrap(), Duration::from_secs(30));
}

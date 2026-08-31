//! The cached-access-token freshness check in the Atlassian mint.
//!
//! This one predicate decides whether a tool call is a decrypt or a trip to
//! Atlassian's token endpoint, and every refresh it lets through is a chance to
//! race another call over a rotating refresh token. Both of its edges matter,
//! and neither is exercised by anything else without live credentials.

use chrono::{Duration, TimeZone as _, Utc};
use systemprompt_web_admin::test_support::{EXPIRY_SKEW_SECS, is_fresh};

fn at(secs_from_now: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000, 0).unwrap() + Duration::seconds(secs_from_now)
}

fn now() -> chrono::DateTime<Utc> {
    at(0)
}

#[test]
fn a_token_with_no_recorded_expiry_is_never_fresh() {
    // Why: the callback writes a credential before its first mint, so a null
    // expiry is a normal state — and treating "unknown" as fresh would hand out
    // an empty or stale token forever.
    assert!(!is_fresh(None, now()));
}

#[test]
fn a_comfortably_future_expiry_is_fresh() {
    assert!(is_fresh(Some(at(3600)), now()));
}

#[test]
fn an_expiry_inside_the_skew_window_is_not_fresh() {
    // Why: a token that expires mid-flight is worse than one refreshed early —
    // the MCP call downstream of the accessor has its own latency budget.
    assert!(!is_fresh(Some(at(EXPIRY_SKEW_SECS - 1)), now()));
    assert!(!is_fresh(Some(at(EXPIRY_SKEW_SECS)), now()));
    assert!(is_fresh(Some(at(EXPIRY_SKEW_SECS + 1)), now()));
}

#[test]
fn an_already_expired_token_is_not_fresh() {
    assert!(!is_fresh(Some(at(-1)), now()));
    assert!(!is_fresh(Some(at(-86_400)), now()));
}

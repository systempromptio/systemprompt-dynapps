//! When Graph asks for a request again, and after how long.
//!
//! A 429 and a 503 are retried on every method: both say the request was not
//! handled. A 504 is retried only on an idempotent method, because a gateway
//! timeout does not say whether the work behind it ran (a `sendMail` that
//! timed out at the gateway may still have gone). All three share one retry
//! budget and one bound on the wait. `Retry-After` is read as delay-seconds
//! or as an HTTP-date (RFC 7231 §7.1.3) in any of its three forms; a date in
//! the past means retry now, and a missing or unreadable header falls back to
//! the configured backoff.

use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::header::RETRY_AFTER;
use reqwest::{Method, StatusCode};

use super::GraphLimits;

// Why: RFC 7231 §7.1.1.1 obliges a recipient to accept the obsolete RFC 850
// and asctime forms of an HTTP-date beside the IMF-fixdate that chrono's
// RFC 2822 parser reads.
const OBSOLETE_HTTP_DATES: [&str; 2] = ["%A, %d-%b-%y %H:%M:%S GMT", "%a %b %e %H:%M:%S %Y"];

pub fn is_retryable(status: StatusCode, method: &Method) -> bool {
    match status {
        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => true,
        StatusCode::GATEWAY_TIMEOUT => method.is_idempotent(),
        _ => false,
    }
}

pub fn parse_retry_after(value: &str, now: DateTime<Utc>) -> Option<Duration> {
    let value = value.trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = http_date(value)?;
    Some((at - now).to_std().unwrap_or(Duration::ZERO))
}

pub(super) fn retry_after(response: &reqwest::Response, limits: &GraphLimits) -> Duration {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| parse_retry_after(value, Utc::now()))
        .unwrap_or(limits.default_backoff)
}

fn http_date(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(date) = DateTime::parse_from_rfc2822(value) {
        return Some(date.with_timezone(&Utc));
    }
    OBSOLETE_HTTP_DATES
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
        .map(|naive| naive.and_utc())
}

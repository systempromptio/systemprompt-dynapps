//! Which Connect answers are worth repeating, and how long to wait between
//! attempts. Pure, so the policy is unit-testable without a server.

use std::time::Duration;

pub const BASE_DELAY: Duration = Duration::from_millis(200);
pub const MAX_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Success,
    Retryable,
    Fatal,
}

#[must_use]
pub const fn classify_status(code: u16) -> StatusClass {
    match code {
        200..=299 => StatusClass::Success,
        429 | 500..=599 => StatusClass::Retryable,
        _ => StatusClass::Fatal,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub retries: u32,
    pub base: Duration,
    pub max: Duration,
}

impl RetryPolicy {
    #[must_use]
    pub const fn new(retries: u32) -> Self {
        Self {
            retries,
            base: BASE_DELAY,
            max: MAX_DELAY,
        }
    }

    #[must_use]
    pub fn delays(&self) -> Vec<Duration> {
        (0..self.retries)
            .map(|attempt| {
                let factor = 1u32.checked_shl(attempt).unwrap_or(u32::MAX);
                self.base.saturating_mul(factor).min(self.max)
            })
            .collect()
    }
}

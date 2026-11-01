//! Wall-clock time for pairing expiry and device timestamps, injected so the
//! expiry rules can be exercised without waiting.

use std::fmt::Debug;

use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync + Debug {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

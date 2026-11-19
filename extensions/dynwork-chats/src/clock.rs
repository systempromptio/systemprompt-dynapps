//! Wall-clock time, injected so the inactivity and pruning rules can be
//! exercised without waiting. The store takes "now" as an argument; callers
//! read it from a [`Clock`].

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

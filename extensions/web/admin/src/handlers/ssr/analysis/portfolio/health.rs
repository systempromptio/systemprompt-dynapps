//! Usage health preserves missing audience and overflow-safe failure
//! thresholds.
use serde::Serialize;
use systemprompt::models::feedback::inventory::InventoryAvailability;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Health {
    Ok,
    Warn,
    Err,
    Muted,
}

impl Health {
    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Err => "err",
            Self::Muted => "muted",
        }
    }
}

/// The measured inputs the health verdict is derived from, separated from the
/// row so the rule can be exercised without an inventory entry.
#[derive(Debug, Clone, Copy)]
pub struct HealthInputs {
    pub availability: InventoryAvailability,
    pub bound: bool,
    pub invocations: i64,
    pub linked_requests: i64,
    pub failed_requests: i64,
    pub users_with_access: Option<usize>,
}

#[must_use]
pub fn health_of(input: &HealthInputs) -> (Health, &'static str) {
    match input.availability {
        InventoryAvailability::Conflicting => return (Health::Err, "conflicting name"),
        InventoryAvailability::Unavailable => return (Health::Err, "unavailable"),
        InventoryAvailability::Withdrawn => return (Health::Err, "withdrawn"),
        InventoryAvailability::Available => {},
    }
    if input.linked_requests > 0
        && i128::from(input.failed_requests) * 10 > i128::from(input.linked_requests)
    {
        return (Health::Err, "failing requests");
    }
    if matches!(input.users_with_access, Some(0)) {
        return (Health::Warn, "no audience");
    }
    if input.invocations == 0 {
        return (Health::Warn, "unused");
    }
    if !input.bound {
        return (Health::Warn, "unbound");
    }
    (Health::Ok, "healthy")
}

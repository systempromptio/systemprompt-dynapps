//! Persistence for the governance record — what actually happened.
//!
//! Every tool-call decision, the policies that produced it, and the rollups the
//! audit pages read are served from here. Rows are append-only history: nothing
//! in this module changes what is allowed, only what was decided.
//!
//! The configured side of that pairing — gateway routes, agent definitions,
//! the access-control YAML — lives in [`super::config`].
//!
//! The `governance-ssr` feature compiles out queries this fork does not
//! serve. `rankings` and `warnings` are exported ungated: the hooks and
//! warnings tabs of `/admin/governance` are this fork's own surface.

/// The window of a paged read: how many rows, starting where.
#[derive(Debug, Clone, Copy)]
pub struct PageSlice {
    pub limit: i64,
    pub offset: i64,
}

impl PageSlice {
    #[must_use]
    pub const fn first(limit: i64) -> Self {
        Self { limit, offset: 0 }
    }
}

/// How a page of decisions is ordered and windowed.
#[derive(Debug, Clone, Copy)]
pub struct DecisionPage {
    pub sort: decision_log::DecisionSort,
    pub slice: PageSlice,
}

pub mod approvals;
pub mod chain;
pub mod counts;
pub mod decision_calls;
pub mod decision_log;
pub mod decisions;
pub mod filter_options;
pub mod findings;
pub mod hook_events;
pub mod rankings;
pub mod resolve;
pub mod secret_audit_log;
pub mod suggest;
pub mod warnings;

#[cfg(feature = "governance-ssr")]
pub use counts::{
    get_governance_counts, get_governance_counts_windowed, list_per_policy_counts,
    list_per_policy_counts_windowed,
};
#[cfg(feature = "governance-ssr")]
pub use decisions::list_decisions_for_policy;
pub use rankings::{list_top_actors, list_top_policies};
pub use warnings::{WarningGroup, WarningGroupBy, WarningRollupRow, group_warning_rollup};

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceCounts {
    pub total: i64,
    pub allowed: i64,
    pub denied: i64,
    pub secret_breaches: i64,
}

#[derive(Debug, Clone)]
pub struct PerPolicyCounts {
    pub policy: String,
    pub allowed: i64,
    pub denied: i64,
    pub last_at: Option<chrono::DateTime<chrono::Utc>>,
}

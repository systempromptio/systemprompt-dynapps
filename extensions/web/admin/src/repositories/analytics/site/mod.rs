//! Read side of the site analytics dashboard (`/admin/analytics`).
//!
//! Every query here takes the same [`SiteScope`] and applies it with a static
//! null-skip bind: NULL spans every user, and otherwise the query narrows to
//! the resolved user id list. Synthetic gateway rows
//! (`ai_requests.synthetic`) are excluded everywhere: demo traffic is not a
//! business metric.

use systemprompt::identifiers::UserId;

use crate::util::scope::SubjectScope;

pub mod anomalies;
pub mod code;
pub mod distribution;
pub mod kpis;
pub mod latency;
pub mod leaderboards;
pub mod model_series;
pub mod series;
pub mod session_costs;
pub mod user_rollups;

/// Drill-down filters, all conjunctive. Written out rather than derived
/// because a default would make the every-user view the one a caller
/// reaches by naming nothing.
#[derive(Debug, Clone)]
pub struct SiteScope {
    pub scope: SubjectScope,
    pub user_id: Option<UserId>,
}

impl SiteScope {
    #[must_use]
    pub const fn new(scope: SubjectScope) -> Self {
        Self {
            scope,
            user_id: None,
        }
    }

    #[must_use]
    pub fn user_id_str(&self) -> Option<&str> {
        self.user_id.as_ref().map(UserId::as_str)
    }
}

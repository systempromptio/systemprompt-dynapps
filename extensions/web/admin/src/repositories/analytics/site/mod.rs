//! Read side of the site analytics dashboard (`/admin/analytics`).
//!
//! Every query here takes the same [`SiteScope`] and applies it with a static
//! null-skip bind: NULL spans every user, and otherwise the query narrows to
//! the resolved user id list. Synthetic gateway rows
//! (`ai_requests.synthetic`) are excluded everywhere: demo traffic is not a
//! business metric.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::scope::{Attribution, Scope, SubjectScope};

pub mod anomalies;
pub mod code;
pub mod cost;
pub mod distribution;
pub mod kpis;
pub mod latency;
pub mod leaderboards;
pub mod model_series;
pub mod models;
pub mod series;
pub mod session_costs;
pub mod sessions;
pub mod skills;
pub mod tools;
pub mod user_rollups;

/// Drill-down filters, all conjunctive. Written out rather than derived
/// because a default would make the every-user view the one a caller
/// reaches by naming nothing.
#[derive(Debug, Clone)]
pub struct SiteScope {
    pub scope: SubjectScope,
    pub user_id: Option<UserId>,
    /// How a person in more than one container is counted. Every total,
    /// leaderboard and cost figure on this page is `Exclusive`, so container
    /// figures partition the instance and sum back to it.
    pub attribution: Attribution,
}

impl SiteScope {
    #[must_use]
    pub const fn new(scope: SubjectScope) -> Self {
        Self {
            scope,
            user_id: None,
            attribution: Attribution::Exclusive,
        }
    }

    /// Turn the scope a reader asked for into the user id list every query
    /// binds. `Exclusive` is the default and the only attribution a total is
    /// ever reported under; a caller asking for `Member` is showing a
    /// "who uses what" breakdown and must label it as overlapping.
    pub async fn resolve(
        pool: &PgPool,
        scope: &Scope,
        attribution: Attribution,
    ) -> Result<Self, sqlx::Error> {
        let resolved = scope.resolve(pool, attribution).await?;
        Ok(Self {
            scope: resolved,
            user_id: match scope {
                Scope::User(user_id) => Some(user_id.clone()),
                _ => None,
            },
            attribution,
        })
    }

    /// Whether the figures on screen deliberately overlap, so the page can
    /// say so rather than letting a reader sum them.
    #[must_use]
    pub const fn is_member_view(&self) -> bool {
        matches!(self.attribution, Attribution::Member)
    }

    #[must_use]
    pub fn user_id_str(&self) -> Option<&str> {
        self.user_id.as_ref().map(UserId::as_str)
    }
}

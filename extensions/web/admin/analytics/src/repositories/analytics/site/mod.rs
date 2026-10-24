//! Read side of the site analytics dashboard (`/admin/analytics`).
//!
//! Every query here takes the same [`SiteScope`] and applies it with a static
//! null-skip bind: NULL spans every user, and otherwise the query narrows to
//! the resolved user id list. Only production gateway rows
//! (`ai_requests.traffic = 'production'`) count: evaluation and demo traffic
//! is not a business metric.
//!
//! Every total, leaderboard and cost figure is reported under `Exclusive`
//! attribution, so container figures partition the instance and sum back to
//! it. A `Member` view is a "who uses what" breakdown whose figures overlap,
//! and the page must label it so.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::repositories::scope::{Attribution, Scope, SubjectScope};

pub mod anomalies;
pub mod cost;
pub mod distribution;
pub mod kpis;
pub mod latency;
pub mod leaderboards;
pub mod model_series;
pub mod models;
pub mod series;
pub mod sessions;
pub mod skills;
pub mod tools;
pub mod user_rollups;

pub async fn resolve_site_scope(
    pool: &PgPool,
    scope: &Scope,
    attribution: Attribution,
) -> Result<SiteScope, sqlx::Error> {
    let resolved = scope.resolve(pool, attribution).await?;
    Ok(SiteScope {
        scope: resolved,
        user_id: match scope {
            Scope::User(user_id) => Some(user_id.clone()),
            _ => None,
        },
        attribution,
    })
}

/// Drill-down filters, all conjunctive. Written out rather than derived
/// because a default would make the every-user view the one a caller
/// reaches by naming nothing.
#[derive(Debug, Clone)]
pub struct SiteScope {
    pub scope: SubjectScope,
    pub user_id: Option<UserId>,
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

    #[must_use]
    pub const fn is_member_view(&self) -> bool {
        matches!(self.attribution, Attribution::Member)
    }

    #[must_use]
    pub fn user_id_str(&self) -> Option<&str> {
        self.user_id.as_ref().map(UserId::as_str)
    }
}

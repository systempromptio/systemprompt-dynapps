//! The header counts for the access-control page.
//!
//! Three totals an auditor reads before the ledger: how many groups, projects
//! and people the rules below can name. Each read degrades to zero rather
//! than taking the page down; the ledger is the reason the page was opened.

use sqlx::PgPool;

use systemprompt_web_admin_core::repositories::scope::SubjectScope;

use super::view::AcStatsView;

pub(super) async fn load_stats(pool: &PgPool) -> AcStatsView {
    let (users, groups, projects) = tokio::join!(
        systemprompt_web_admin_core::repositories::principal::listing::list_users(
            pool,
            &SubjectScope::All
        ),
        systemprompt_web_admin_core::repositories::scope::containers::list_group_options(pool),
        systemprompt_web_admin_core::repositories::scope::containers::list_project_summaries(pool),
    );
    AcStatsView {
        users: users
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: user listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
        groups: groups
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: group listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
        projects: projects
            .inspect_err(|e| tracing::warn!(error = %e, "access-control: project listing failed"))
            .map(|v| v.len())
            .unwrap_or_default(),
    }
}

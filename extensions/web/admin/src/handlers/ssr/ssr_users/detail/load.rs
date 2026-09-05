//! Per-tab loading for the user detail page.
//!
//! Every function here loads exactly one tab. A failed read degrades to an
//! empty panel rather than taking the page down: the header already answers
//! "who is this account", which is most of why the page was opened.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::repositories::users::enrolment::{UserCommitRow, UserDeviceRow};
use crate::repositories::users::sessions::SigninSessionRow;

// Why: how many commits the Usage tab lists before deferring to the sessions
// pages.
pub(super) const COMMIT_LIMIT: i64 = 20;

// Why: how many sign-in sessions one page of the Sessions tab carries.
pub(super) const SESSION_PAGE_SIZE: i64 = 50;

pub(super) struct IdentityData {
    pub roles: Vec<String>,
    pub adfs_groups: Vec<String>,
    pub identities: Vec<repositories::users::federated::LinkedIdentityRow>,
    pub salesforce_username: Option<String>,
    pub share_token_version: i32,
}

pub(super) async fn load_identity(
    pool: &PgPool,
    user_id: &UserId,
    roles: &[String],
) -> IdentityData {
    let (adfs, identities, salesforce, share) = tokio::join!(
        repositories::groups::members::list_source_ad_groups(pool, user_id),
        repositories::users::federated::list_linked_identities(pool, user_id),
        repositories::users::salesforce_identity::find_username(pool, user_id),
        repositories::users::share_token::find_share_token_version(pool, user_id),
    );
    IdentityData {
        roles: roles.to_vec(),
        adfs_groups: warn_empty(adfs, "AD groups"),
        identities: warn_empty(identities, "federated identities"),
        salesforce_username: salesforce.unwrap_or_default(),
        share_token_version: share.unwrap_or_default().unwrap_or(0),
    }
}

pub(super) struct MembershipData {
    pub groups: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub projects: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub defaults: Option<repositories::scope::defaults::ScopeDefaults>,
}

pub(super) async fn load_membership(pool: &PgPool, user_id: &UserId) -> MembershipData {
    let (groups, projects, defaults) = tokio::join!(
        super::super::scope_data::group_choices(pool, user_id),
        super::super::scope_data::project_choices(pool, user_id),
        repositories::scope::defaults::find_scope_defaults(pool, user_id),
    );
    MembershipData {
        groups,
        projects,
        defaults: defaults
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: scope defaults unavailable"))
            .unwrap_or_default(),
    }
}

pub(super) async fn load_devices(pool: &PgPool, user_id: &UserId) -> Vec<UserDeviceRow> {
    warn_empty(
        repositories::users::enrolment::list_user_devices(pool, user_id).await,
        "devices",
    )
}

pub(super) async fn load_sessions(pool: &PgPool, user_id: &UserId) -> Vec<SigninSessionRow> {
    warn_empty(
        repositories::users::sessions::list_signin_sessions(pool, user_id).await,
        "sign-in sessions",
    )
}

pub(super) struct UsageData {
    pub summary: repositories::users::usage::UserGatewayUsage,
    pub models: Vec<repositories::users::usage::ModelShare>,
    pub conversations: Vec<crate::handlers::ssr::types::UserConversationView>,
    pub commits: Vec<UserCommitRow>,
}

// Why: the former `/admin/analytics/users/{id}` page, folded in as a tab. The
// same four reads it made, minus the site-wide KPI block, which asked the
// instance a question this page has already narrowed to one person.
pub(super) async fn load_usage(pool: &PgPool, user_id: &UserId) -> UsageData {
    let (summary, models, conversations, commits) = tokio::join!(
        repositories::users::usage::get_ai_request_summary(pool, user_id),
        repositories::users::usage::list_top_models(pool, user_id, None, 10),
        super::super::data::load_conversations(pool, user_id),
        repositories::users::enrolment::list_user_commits(pool, user_id, COMMIT_LIMIT),
    );
    UsageData {
        summary: summary
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: usage summary failed"))
            .unwrap_or_default(),
        models: warn_empty(models, "top models"),
        conversations,
        commits: warn_empty(commits, "commits"),
    }
}

fn warn_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.inspect_err(|e| tracing::warn!(error = %e, panel = what, "user detail: panel unavailable"))
        .unwrap_or_default()
}

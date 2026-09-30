//! Per-tab loading for the user detail page.
//!
//! Every function here loads exactly one tab. A failed read is recorded on
//! the page's `Faults` and its panel renders empty rather than taking the
//! page down: the header already answers "who is this account", which is
//! most of why the page was opened.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::repositories::users::enrolment::UserDeviceRow;
use crate::repositories::users::sessions::SigninSessionRow;
use systemprompt_web_admin_core::repositories::conversations::rows::{
    ConversationFilter, ConversationRow, ConversationTotals, find_latest_conversation,
    get_conversation_totals,
};
use systemprompt_web_admin_core::repositories::conversations::{
    HistoryFilter, HistoryItem, list_history_items,
};
use systemprompt_web_admin_core::ssr::faults::Faults;

pub(super) const CONVERSATION_PAGE_SIZE: i64 = 25;

pub(super) struct UserConversationsData {
    pub items: Vec<HistoryItem>,
    pub total: i64,
}

pub(super) async fn load_conversations(
    pool: &PgPool,
    user_id: &UserId,
    page: i64,
    faults: &mut Faults,
) -> UserConversationsData {
    let scope = [user_id.as_str().to_owned()];
    let listed = list_history_items(
        pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: false,
            since: None,
            until: None,
        },
        CONVERSATION_PAGE_SIZE,
        page * CONVERSATION_PAGE_SIZE,
    )
    .await;
    let (items, total) = faults.take("conversations", listed);
    UserConversationsData { items, total }
}

pub(super) struct IdentityData {
    pub roles: Vec<String>,
    pub adfs_groups: Vec<String>,
    pub identities: Vec<repositories::users::federated::LinkedIdentityRow>,
    pub salesforce_identities: Vec<systemprompt_web_admin_platform::repositories::connectors::salesforce_identity::SalesforceIdentity>,
    pub share_token_version: i32,
    pub manual_roles_valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

pub(super) async fn load_identity(
    pool: &PgPool,
    user_id: &UserId,
    roles: &[String],
    faults: &mut Faults,
) -> IdentityData {
    let (adfs, identities, salesforce, share, until) = tokio::join!(
        systemprompt_web_admin_core::repositories::principal::group_membership::list_source_ad_groups(pool, user_id),
        repositories::users::federated::list_linked_identities(pool, user_id),
        systemprompt_web_admin_platform::repositories::connectors::salesforce_identity::list_identities(pool, user_id),
        systemprompt_web_admin_core::repositories::share_token::find_share_token_version(
            pool, user_id
        ),
        repositories::users::roles::find_manual_roles_valid_until(pool, user_id),
    );
    IdentityData {
        roles: roles.to_vec(),
        adfs_groups: faults.take("AD groups", adfs),
        identities: faults.take("federated identities", identities),
        salesforce_identities: faults.take("Salesforce identities", salesforce),
        share_token_version: faults.take("share token", share).unwrap_or(0),
        manual_roles_valid_until: faults.take("role expiry", until),
    }
}

pub(super) struct MembershipData {
    pub groups: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub projects: Vec<crate::handlers::ssr::types::MembershipChoiceView>,
    pub defaults: Option<systemprompt_web_admin_core::repositories::scope::defaults::ScopeDefaults>,
}

pub(super) async fn load_membership(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> MembershipData {
    let groups = super::super::scope_data::group_choices(pool, user_id, faults).await;
    let projects = super::super::scope_data::project_choices(pool, user_id, faults).await;
    let defaults = systemprompt_web_admin_core::repositories::scope::defaults::find_scope_defaults(
        pool, user_id,
    )
    .await;
    MembershipData {
        groups,
        projects,
        defaults: faults.take("scope defaults", defaults),
    }
}

pub(super) struct AccessData {
    pub overview: super::access_overview::AccessOverview,
    pub rules_available: bool,
    pub matrix: Option<systemprompt_web_admin_core::repositories::entitlement::UserMatrix>,
    pub rules: Vec<systemprompt_web_admin_core::types::access_control::AccessControlRule>,
}

pub(super) async fn load_access(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> AccessData {
    let overview = super::access_overview::load(pool, user_id).await;
    let services_path = match systemprompt_web_admin_core::request::get_services_path() {
        Ok(path) => path,
        Err(error) => {
            faults.record("access matrix", &error);
            return AccessData {
                overview,
                rules_available: false,
                matrix: None,
                rules: Vec::new(),
            };
        },
    };
    let sections = crate::handlers::access_control::build_matrix_sections(&services_path);
    let (matrix, rules) = tokio::join!(
        systemprompt_web_admin_core::repositories::entitlement::resolve_user_matrix(
            pool, user_id, sections
        ),
        systemprompt_web_admin_core::repositories::entitlement::list_all_rules(pool),
    );
    AccessData {
        overview,
        rules_available: rules.is_ok(),
        matrix: faults.take("access matrix", matrix),
        rules: faults.take("access rules", rules),
    }
}

pub(super) async fn load_devices(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> Vec<UserDeviceRow> {
    faults.take(
        "devices",
        repositories::users::enrolment::list_user_devices(pool, user_id).await,
    )
}

pub(super) async fn load_sessions(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> Vec<SigninSessionRow> {
    faults.take(
        "sign-in sessions",
        repositories::users::sessions::list_signin_sessions(pool, user_id).await,
    )
}

pub(super) struct UsageData {
    pub summary: systemprompt_web_admin_core::repositories::user_usage::UserGatewayUsage,
    pub models: Vec<systemprompt_web_admin_core::repositories::user_usage::ModelShare>,
    pub latest: Option<ConversationRow>,
    pub totals: ConversationTotals,
}


pub(super) async fn load_usage(pool: &PgPool, user_id: &UserId, faults: &mut Faults) -> UsageData {
    let filter = ConversationFilter {
        user_id: Some(user_id.clone()),
        include_side_calls: true,
        ..ConversationFilter::default()
    };
    let (summary, models, latest, totals) = tokio::join!(
        systemprompt_web_admin_core::repositories::user_usage::get_ai_request_summary(
            pool, user_id
        ),
        systemprompt_web_admin_core::repositories::user_usage::list_top_models(
            pool, user_id, None, 10
        ),
        find_latest_conversation(pool, user_id),
        get_conversation_totals(pool, &filter),
    );
    UsageData {
        summary: faults.take("usage summary", summary),
        models: faults.take("top models", models),
        latest: faults.take("latest conversation", latest),
        totals: faults.take("conversation totals", totals),
    }
}

pub(super) struct Headline {
    pub(super) profile:
        Option<systemprompt_web_admin_core::repositories::principal::role::UserAccessProfile>,
    pub(super) summary: systemprompt_web_admin_core::repositories::user_usage::UserGatewayUsage,
    pub(super) defaults:
        Option<systemprompt_web_admin_core::repositories::scope::defaults::ScopeDefaults>,
}

pub(super) async fn load_headline(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> Headline {
    let (profile, summary, defaults) = tokio::join!(
        systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile(
            pool, user_id
        ),
        systemprompt_web_admin_core::repositories::user_usage::get_ai_request_summary(
            pool, user_id
        ),
        systemprompt_web_admin_core::repositories::scope::defaults::find_scope_defaults(
            pool, user_id
        ),
    );
    Headline {
        profile: faults.take("access profile", profile),
        summary: faults.take("request summary", summary),
        defaults: faults.take("scope defaults", defaults),
    }
}

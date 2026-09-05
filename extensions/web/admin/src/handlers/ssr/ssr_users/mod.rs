//! `/admin/users` user roster (grouped by group) and the
//! per-user detail page.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::repositories::scope::ScopeRequest;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext, UsersQuery};
use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use super::types::{PageStatView, UserDetailPageData, UsersPageData};

mod data;
mod scope_data;
mod view;

pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UsersQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let request =
        ScopeRequest::from_query(&user_ctx, query.group.as_deref(), query.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let users = repositories::users::queries::list_users(&pool, &scope)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list users");
            vec![]
        });

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();

    let mut groups = data::load_user_groups(&pool, &users).await;
    scope_data::attach_project_ids(&pool, &mut groups).await;
    let (group_names, project_names) = scope_data::load_scope_names(&pool).await;

    let page_stats = vec![
        PageStatView {
            value: total_users as i64,
            label: "Users",
        },
        PageStatView {
            value: active_users as i64,
            label: "Active",
        },
        PageStatView {
            value: total_events,
            label: "Events",
        },
    ];

    let data = UsersPageData {
        page: "users",
        title: "Users",
        groups,
        scope_filter: super::list_view::scope_filter_view(
            &pool,
            &user_ctx,
            &request,
            "/admin/users",
            Vec::new(),
        )
        .await,
        group_names,
        project_names,
        total_users,
        active_users,
        total_events,
        page_stats,
    };

    Ok(super::render_typed_page(
        &engine, "users", &data, &user_ctx, &mkt_ctx,
    ))
}

// Why: the roster links a user by path (`/admin/users/{id}`); the header
// search and a handful of API callers still resolve to `?id=`. Both forms are
// the same page, so they share one body and differ only in where the id is
// read from.
pub(crate) async fn user_detail_by_id_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> AdminHtmlResult<Response> {
    render_user_detail(&user_ctx, &mkt_ctx, &engine, &pool, Some(user_id.as_str())).await
}

pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> AdminHtmlResult<Response> {
    render_user_detail(&user_ctx, &mkt_ctx, &engine, &pool, params.id()).await
}

async fn render_user_detail(
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
    engine: &AdminTemplateEngine,
    pool: &PgPool,
    requested_id: Option<&str>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console && Some(user_ctx.user_id.as_str()) != requested_id {
        return Err(AdminError::Forbidden("You can only view your own profile.".to_owned()).into());
    }

    let Some(id) = requested_id else {
        let data = blank_user_detail();
        return Ok(super::render_typed_page(
            engine,
            "user-detail",
            &data,
            user_ctx,
            mkt_ctx,
        ));
    };
    let user_id = UserId::new(id);

    // Why: `Err` and `Ok(None)` must not collapse together here: this value alone
    // decides whether the page renders "User not found.", so a failed query
    // would tell an admin the account had been deleted. Only a genuine absence
    // is a not-found.
    let detail = repositories::users::queries::find_user_detail(pool, &user_id).await?;
    let gamification: Option<crate::types::UserGamificationProfile> = None;

    if detail.is_none() {
        return Err(AdminError::NotFound("No such user.".to_owned()).into());
    }

    let DetailParts {
        memberships,
        user_assignments,
        user_devices,
        user_devices_count,
        effective,
        runtime,
        ai_usage,
        conversations,
        user_adfs_groups,
    } = load_detail_parts(pool, detail.as_ref(), &user_id).await?;

    let has_effective_permissions = effective
        .as_ref()
        .is_some_and(|eff| !eff.gateway_routes.is_empty() || !eff.mcp_servers.is_empty());

    let held_roles = detail.as_ref().map(|d| d.roles.clone()).unwrap_or_default();
    let data = UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: detail,
        gamification,
        not_found: false,
        user_groups: memberships.groups.join(", "),
        user_projects: memberships.projects.join(", "),
        user_adfs_groups,
        group_choices: scope_data::group_choices(pool, &user_id).await,
        project_choices: scope_data::project_choices(pool, &user_id).await,
        role_choices: scope_data::role_choices(&held_roles),
        can_edit_membership: user_ctx.is_admin,
        // Why: an admin must not be able to drop their own last role from this
        // form — the API refuses it, and offering the control anyway turns a
        // guardrail into an error toast.
        is_self: user_ctx.user_id.as_str() == user_id.as_str(),
        user_assignments,
        user_devices,
        user_devices_count,
        runtime,
        has_conversations: !conversations.is_empty(),
        conversations_url: format!(
            "/admin/contexts?user_id={}",
            urlencoding::encode(user_id.as_str())
        ),
        ai_usage,
        conversations,
        effective_permissions: effective,
        has_effective_permissions,
    };
    Ok(super::render_typed_page(
        engine,
        "user-detail",
        &data,
        user_ctx,
        mkt_ctx,
    ))
}

struct DetailParts {
    memberships: data::UserMemberships,
    user_assignments: super::types::UserAssignmentSummary,
    user_devices: Vec<super::types::UserDeviceView>,
    user_devices_count: i64,
    effective: Option<repositories::governance::effective::EffectivePermissions>,
    runtime: Option<super::types::UserRuntimeView>,
    ai_usage: Option<super::types::UserAiUsageView>,
    conversations: Vec<super::types::UserConversationView>,
    user_adfs_groups: Vec<String>,
}

// Why: everything the detail page shows beside the user row itself, gathered in
// one place so the handler stays a page assembly rather than a query list.
async fn load_detail_parts(
    pool: &PgPool,
    detail: Option<&crate::types::UserDetail>,
    user_id: &UserId,
) -> AdminHtmlResult<DetailParts> {
    let (memberships, user_assignments, user_devices, user_devices_count, effective) = match detail
    {
        Some(d) => data::collect_user_detail_extras(pool, d).await?,
        None => (
            data::UserMemberships::default(),
            super::types::UserAssignmentSummary::default(),
            Vec::new(),
            0,
            None,
        ),
    };
    let (runtime, ai_usage, conversations) = match detail {
        Some(d) => data::load_user_activity(pool, d).await,
        None => (None, None, Vec::new()),
    };
    // Why: a directory read that fails must not fail the page; the AD group
    // names are context, the group membership is the fact.
    let user_adfs_groups = repositories::groups::members::list_source_ad_groups(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: AD group lookup failed"))
        .unwrap_or_default();
    Ok(DetailParts {
        memberships,
        user_assignments,
        user_devices,
        user_devices_count,
        effective,
        runtime,
        ai_usage,
        conversations,
        user_adfs_groups,
    })
}

fn blank_user_detail() -> UserDetailPageData {
    UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: None,
        gamification: None,
        not_found: true,
        user_groups: String::new(),
        user_projects: String::new(),
        user_adfs_groups: Vec::new(),
        group_choices: Vec::new(),
        project_choices: Vec::new(),
        role_choices: Vec::new(),
        can_edit_membership: false,
        is_self: false,
        user_assignments: super::types::UserAssignmentSummary::default(),
        user_devices: Vec::new(),
        user_devices_count: 0,
        runtime: None,
        ai_usage: None,
        conversations: Vec::new(),
        has_conversations: false,
        conversations_url: String::new(),
        effective_permissions: None,
        has_effective_permissions: false,
    }
}

//! `/admin/access/users` user roster (grouped by project) and the
//! per-user detail page.

use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext, UsersQuery};
use crate::util::project_scope::ProjectScope;
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use super::types::{PageStatView, UserDetailPageData, UserRuntimeView, UsersPageData};

mod data;
mod view;

pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UsersQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let scope = ProjectScope::resolve(&user_ctx, query.project.as_deref());
    let users = repositories::users::queries::list_users(&pool, &scope)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list users");
            vec![]
        });

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();

    let groups = data::load_user_groups(&pool, &users).await;

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
        project_filter: super::list_view::project_filter_view(
            &user_ctx,
            scope,
            "/admin/access/users",
            Vec::new(),
        ),
        total_users,
        active_users,
        total_events,
        page_stats,
    };

    Ok(super::render_typed_page(
        &engine, "users", &data, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin && Some(user_ctx.user_id.as_str()) != params.id() {
        return Err(AdminError::Forbidden("You can only view your own profile.".to_owned()).into());
    }

    let Some(id) = params.id() else {
        let data = blank_user_detail();
        return Ok(super::render_typed_page(
            &engine,
            "user-detail",
            &data,
            &user_ctx,
            &mkt_ctx,
        ));
    };
    let user_id = UserId::new(id);

    // Why: `Err` and `Ok(None)` must not collapse together here: this value alone
    // decides whether the page renders "User not found.", so a failed query
    // would tell an admin the account had been deleted. Only a genuine absence
    // is a not-found.
    let detail = repositories::users::queries::find_user_detail(&pool, &user_id).await?;
    let gamification: Option<crate::types::UserGamificationProfile> = None;

    let not_found = detail.is_none();

    let (user_project, user_assignments, user_devices, user_devices_count, effective) =
        match detail.as_ref() {
            Some(d) => data::collect_user_detail_extras(&pool, d).await?,
            None => (
                None,
                super::types::UserAssignmentSummary::default(),
                Vec::new(),
                0,
                None,
            ),
        };

    let runtime = match detail.as_ref() {
        Some(d) => load_runtime_view(&pool, d).await,
        None => None,
    };

    // Why: a group read that fails must not fail the page; the groups are
    // context, the project is the fact.
    let user_adfs_groups = repositories::users::adfs_groups::list_groups(&pool, &user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: AD group lookup failed"))
        .unwrap_or_default();

    let has_effective_permissions = effective
        .as_ref()
        .is_some_and(|eff| !eff.gateway_routes.is_empty() || !eff.mcp_servers.is_empty());

    let data = UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: detail,
        gamification,
        not_found,
        user_project: user_project
            .map_or_else(|| "Unassigned".to_owned(), |p| p.label().to_owned()),
        user_adfs_groups,
        user_assignments,
        user_devices,
        user_devices_count,
        runtime,
        effective_permissions: effective,
        has_effective_permissions,
    };
    Ok(super::render_typed_page(
        &engine,
        "user-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn blank_user_detail() -> UserDetailPageData {
    UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: None,
        gamification: None,
        not_found: true,
        user_project: String::new(),
        user_adfs_groups: Vec::new(),
        user_assignments: super::types::UserAssignmentSummary::default(),
        user_devices: Vec::new(),
        user_devices_count: 0,
        runtime: None,
        effective_permissions: None,
        has_effective_permissions: false,
    }
}

async fn load_runtime_view(pool: &PgPool, d: &crate::types::UserDetail) -> Option<UserRuntimeView> {
    repositories::users::queries::get_user_runtime_detail(pool, &d.user_id)
        .await
        .ok()
        .map(|r| UserRuntimeView {
            connected_agents: r.connected_agents,
            total_agents: r.total_agents,
            tokens_in: r.tokens_in,
            tokens_out: r.tokens_out,
            last_bridge_version: r.last_bridge_version,
            last_os: r.last_os,
            last_hostname: r.last_hostname,
            last_heartbeat_at: r.last_heartbeat_at.map(|t| t.to_rfc3339()),
        })
}

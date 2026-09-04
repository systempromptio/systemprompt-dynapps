//! Data collection for the user roster and detail pages.
//!
//! Fans out the repository / runtime queries each page needs and shapes the raw
//! rows just enough for the view layer: roster aggregates, per-user assignment
//! and device extras.

use sqlx::PgPool;

use crate::repositories;
use crate::services::marketplaces::load_marketplaces;
use crate::types::Project;

use super::super::types::{ProjectGroup, UserAssignmentSummary, UserDeviceView};
use super::view;

type UserDetailExtras = (
    Option<Project>,
    UserAssignmentSummary,
    Vec<UserDeviceView>,
    i64,
    Option<repositories::governance::effective::EffectivePermissions>,
);

pub(super) async fn load_user_groups(
    pool: &PgPool,
    users: &[crate::types::UserSummary],
) -> Vec<ProjectGroup> {
    let aggregates = repositories::users::aggregates::list_user_management_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user management aggregates");
            Vec::new()
        });

    let runtime = repositories::users::queries::list_user_runtime_aggregates(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch user runtime aggregates");
            Vec::new()
        });

    // Why: an empty list is not a safe default here: `resolve_marketplaces`
    // seeds from every YAML marketplace and *subtracts* the deny rows, so
    // losing the overrides does not lose grants — it loses the denials, and
    // every explicitly denied marketplace renders as granted. That is the one
    // failure direction on this page that over-reports access, so it degrades
    // to showing nothing rather than to showing everything.
    let (overrides, overrides_failed) =
        match repositories::users::aggregates::list_user_marketplace_overrides(pool).await {
            Ok(rows) => (rows, false),
            Err(e) => {
                tracing::warn!(error = %e, "Failed to fetch marketplace overrides");
                (Vec::new(), true)
            },
        };

    let yaml_marketplaces: Vec<(String, String)> = if overrides_failed {
        Vec::new()
    } else {
        load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name))
            .collect()
    };

    let enriched_users =
        view::enrich_users(users, &aggregates, &runtime, &overrides, &yaml_marketplaces);
    view::group_by_project(enriched_users)
}

pub(super) async fn collect_user_detail_extras(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> crate::error::AdminHtmlResult<UserDetailExtras> {
    // Why: `roles` is fed to `compute_effective_permissions`, so an empty
    // value would render ALLOW/DENY rows under a caption promising they were
    // computed against this user's actual roles.
    let (roles, project) = repositories::users::queries::find_user_roles_project(pool, &d.user_id)
        .await?
        .unwrap_or_else(|| (Vec::new(), None));

    let mut assignments = UserAssignmentSummary::default();
    let devices_count = if let Ok(rows) =
        repositories::users::aggregates::list_user_management_aggregates(pool).await
        && let Some(row) = rows.into_iter().find(|r| r.user_id == d.user_id.as_str())
    {
        assignments.skills_count = row.assigned_skills_count;
        row.devices_count
    } else {
        0i64
    };

    // Why: Same fail-closed reasoning as the roster: losing the overrides loses the
    // *denials*, so an explicitly denied marketplace would render as granted.
    // Show none rather than all.
    let (user_overrides, overrides_failed): (
        Vec<repositories::users::aggregates::UserMarketplaceOverride>,
        bool,
    ) = match repositories::users::aggregates::list_user_marketplace_overrides(pool).await {
        Ok(rows) => (
            rows.into_iter()
                .filter(|o| o.user_id == d.user_id.as_str())
                .collect(),
            false,
        ),
        Err(e) => {
            tracing::warn!(error = %e, user_id = %d.user_id, "Failed to fetch marketplace overrides");
            (Vec::new(), true)
        },
    };
    let yaml_marketplaces: Vec<(String, String)> = if overrides_failed {
        Vec::new()
    } else {
        load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name))
            .collect()
    };
    let override_refs: Vec<&repositories::users::aggregates::UserMarketplaceOverride> =
        user_overrides.iter().collect();
    assignments.marketplaces = view::resolve_marketplaces(&yaml_marketplaces, &override_refs);
    assignments.marketplaces_count = assignments.marketplaces.len() as i64;

    let devices = collect_user_devices(pool, d).await;

    let effective = Some(
        repositories::governance::effective::compute_effective_permissions(
            pool, &d.user_id, &roles,
        )
        .await,
    );

    Ok((project, assignments, devices, devices_count, effective))
}

async fn collect_user_devices(pool: &PgPool, d: &crate::types::UserDetail) -> Vec<UserDeviceView> {
    let Ok(pats) = repositories::bridge::list_api_keys_for_user(pool, &d.user_id).await else {
        return Vec::new();
    };
    let app_links: std::collections::HashMap<
        String,
        (String, String, Option<chrono::DateTime<chrono::Utc>>),
    > = repositories::users::devices::list_device_app_links(pool, &d.user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: load device app_links failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|r| (r.device_id, (r.app_platform, r.app_version, r.last_seen_at)))
        .collect();

    pats.into_iter()
        .map(|row| {
            let link = app_links.get(&row.id);
            UserDeviceView {
                id: row.id,
                name: row.name,
                key_prefix: row.key_prefix,
                platform: link.map(|(p, _, _)| p.clone()),
                app_version: link.map(|(_, v, _)| v.clone()).filter(|v| !v.is_empty()),
                last_seen_at: link.and_then(|(_, _, ts)| *ts).or(row.last_used_at),
                revoked: row.revoked_at.is_some(),
            }
        })
        .collect()
}

// Why: `ai_requests` is where gateway (`/v1/messages`) traffic lands, and it is
// the only place a user who never runs Claude Code appears at all. A failure
// here degrades to no card rather than failing the page: the rest of the user
// detail is still the answer to "who is this account".
pub(super) async fn load_ai_usage(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Option<super::super::types::UserAiUsageView> {
    let summary = repositories::users::usage::get_ai_request_summary(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: AI request summary failed"))
        .ok()?;
    let models = repositories::users::usage::list_top_models(pool, user_id, None, 5)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: top models failed"))
        .unwrap_or_default();
    Some(view::ai_usage_view(&summary, &models))
}

// Why: no project filter — this is the admin's view of one named account, so
// the project scoping that guards the cross-user contexts list has nothing to
// add and would only hide rows from an admin who asked for them by id.
pub(super) async fn load_conversations(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Vec<super::super::types::UserConversationView> {
    let filter = repositories::analytics::contexts_list::ContextListFilter {
        user_id: Some(user_id.clone()),
        limit: 50,
        ..Default::default()
    };
    repositories::analytics::contexts_list::list_context_list(pool, &filter)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "ssr_users: conversation list failed"))
        .unwrap_or_default()
        .iter()
        .map(view::conversation_view)
        .collect()
}

// Why: the three per-user activity reads are independent, so they run
// concurrently and are collected here rather than in the handler, which the
// line-count lint holds to a page-assembly role.
pub(super) async fn load_user_activity(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> (
    Option<super::super::types::UserRuntimeView>,
    Option<super::super::types::UserAiUsageView>,
    Vec<super::super::types::UserConversationView>,
) {
    tokio::join!(
        load_runtime_view(pool, d),
        load_ai_usage(pool, &d.user_id),
        load_conversations(pool, &d.user_id),
    )
}

async fn load_runtime_view(
    pool: &PgPool,
    d: &crate::types::UserDetail,
) -> Option<super::super::types::UserRuntimeView> {
    repositories::users::queries::get_user_runtime_detail(pool, &d.user_id)
        .await
        .ok()
        .map(|r| super::super::types::UserRuntimeView {
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

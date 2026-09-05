//! Reads behind the project listing and detail pages.

use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::types::UserContext;
use crate::types::projects::{ProjectAdMappingRow, ProjectMemberRow};

use crate::repositories::people_usage::breakdown::{
    LinkedScopeRow, ModelUsageRow, count_linked_scopes, list_linked_scopes, list_scope_top_models,
    list_scope_top_skills,
};
use crate::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, LEADERBOARD_LIMIT, MemberUsageRow, Scope, ScopeUsageRow, get_scope_usage,
    list_member_usage,
};

use super::super::people_view::or_default;
use super::super::types::{ProjectRowView, UserOptionView};

pub(super) struct ProjectDetailData {
    pub usage: ScopeUsageRow,
    pub members: Vec<ProjectMemberRow>,
    pub member_usage: HashMap<String, MemberUsageRow>,
    pub active: HashSet<String>,
    pub addable: Vec<UserOptionView>,
    pub groups: Vec<LinkedScopeRow>,
    pub models: Vec<ModelUsageRow>,
    pub skills: Vec<(String, i64)>,
    pub mappings: Vec<ProjectAdMappingRow>,
    pub rules: Vec<crate::types::access_control::AccessControlRule>,
}

pub(super) async fn load_project_rows(pool: &PgPool) -> Vec<ProjectRowView> {
    let summaries = repositories::projects::crud::list_project_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "project summaries failed"))
        .unwrap_or_default();

    let groups = or_default(
        "project group counts",
        count_linked_scopes(pool, Scope::Project).await,
    );

    summaries
        .into_iter()
        .map(|p| ProjectRowView {
            href: format!("/admin/access/projects/{}", p.id),
            group_count: groups.get(&p.id).copied().unwrap_or(0),
            member_count: p.member_count,
            active_members_30d: p.active_members_30d,
            requests_30d: p.requests_30d,
            cost_30d_microdollars: p.cost_30d_microdollars,
            id: p.id,
            name: p.name,
            description: p.description,
        })
        .collect()
}

pub(super) async fn load_project_detail(
    pool: &PgPool,
    project_id: &str,
    user_ctx: &UserContext,
) -> ProjectDetailData {
    let members = repositories::projects::members::list_project_members(pool, project_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "project members failed"))
        .unwrap_or_default();

    let (usage, member_usage, groups, models, skills) = tokio::join!(
        get_scope_usage(pool, Scope::Project, project_id, DEFAULT_WINDOW_DAYS),
        list_member_usage(pool, Scope::Project, project_id, DEFAULT_WINDOW_DAYS),
        list_linked_scopes(pool, Scope::Project, project_id),
        list_scope_top_models(
            pool,
            Scope::Project,
            project_id,
            DEFAULT_WINDOW_DAYS,
            LEADERBOARD_LIMIT
        ),
        list_scope_top_skills(
            pool,
            Scope::Project,
            project_id,
            DEFAULT_WINDOW_DAYS,
            LEADERBOARD_LIMIT
        ),
    );
    let usage = or_default("project usage", usage);
    let member_usage = or_default("project member usage", member_usage);
    let groups = or_default("project groups", groups);
    let models = or_default("project model mix", models);
    let skills: Vec<(String, i64)> = or_default("project top skills", skills)
        .into_iter()
        .map(|s| (s.skill, s.invocations))
        .collect();

    let mappings = repositories::projects::mappings::list_project_ad_mappings(pool, project_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "project AD mappings failed"))
        .unwrap_or_default();

    // Why: only the rules bound to this project are shown, and the page says
    // so — a project is an ACL subject, not a container of entitlement, so
    // listing everything a member happens to reach would misattribute grants
    // that came from their group.
    let rules = repositories::users::access_control::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access rules failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.rule_type.as_str() == "project" && r.rule_value == project_id)
        .collect();

    let addable = if user_ctx.is_admin {
        load_addable_users(pool, &members).await
    } else {
        Vec::new()
    };

    ProjectDetailData {
        usage,
        member_usage,
        active: load_active_user_ids(pool).await,
        addable,
        groups,
        models,
        skills,
        mappings,
        rules,
        members,
    }
}

async fn load_active_user_ids(pool: &PgPool) -> HashSet<String> {
    sqlx::query_scalar::<_, String>("SELECT id FROM users WHERE is_active = true")
        .fetch_all(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "active user ids failed"))
        .unwrap_or_default()
        .into_iter()
        .collect()
}

async fn load_addable_users(pool: &PgPool, members: &[ProjectMemberRow]) -> Vec<UserOptionView> {
    let existing: HashSet<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    sqlx::query_as::<_, (String, Option<String>, String)>(
        "SELECT id, COALESCE(display_name, full_name, name), email
         FROM users
         WHERE NOT ('anonymous' = ANY(roles))
         ORDER BY COALESCE(display_name, full_name, name, id)",
    )
    .fetch_all(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "addable user list failed"))
    .unwrap_or_default()
    .into_iter()
    .filter(|(id, _, _)| !existing.contains(id.as_str()))
    .map(|(id, name, email)| UserOptionView {
        label: name.map_or_else(|| email.clone(), |n| format!("{n} ({email})")),
        user_id: UserId::new(id),
    })
    .collect()
}

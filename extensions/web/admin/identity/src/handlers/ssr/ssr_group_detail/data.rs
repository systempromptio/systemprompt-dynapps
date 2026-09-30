//! Per-tab loaders for the group detail page.
//!
//! Only the active tab's data is fetched. The counts on the tab labels are the
//! exception — they are three cheap reads that must be right on every tab, or
//! the strip would advertise an empty tab as full.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use systemprompt_web_admin_core::catalog_index::marketplaces::load_marketplaces;
use systemprompt_web_admin_core::types::groups::{GroupAdMappingRow, GroupMemberRow};

use crate::repositories::people_usage::attribution::{
    AttributionSourceRow, list_scope_attribution_sources,
};
use crate::repositories::people_usage::breakdown::{
    LinkedScopeRow, ModelUsageRow, list_linked_scopes, list_scope_top_models,
    list_scope_top_skills, list_scope_top_tools,
};
use crate::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, LEADERBOARD_LIMIT, MemberUsageRow, ScopeUsageRow, get_scope_usage,
    list_daily_requests, list_member_usage,
};
use systemprompt_web_admin_core::repositories::scope::{Attribution, ScopeQuery, ScopeTarget};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_shared::GroupId;

use super::super::types::{AccessSectionView, UserOptionView};
use super::tabs::TabCounts;

pub(super) struct MembersData {
    pub rows: Vec<GroupMemberRow>,
    pub usage: std::collections::HashMap<String, MemberUsageRow>,
    pub active: std::collections::HashSet<String>,
    pub addable: Vec<UserOptionView>,
}

pub(super) struct UsageTabData {
    pub attribution: Vec<AttributionSourceRow>,
    pub models: Vec<ModelUsageRow>,
    pub daily: Vec<i64>,
    pub skills: Vec<(String, i64)>,
    pub tools: Vec<(String, i64)>,
    pub leaderboard: Vec<MemberUsageRow>,
}

pub(super) async fn load_counts(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> TabCounts {
    let q = member_query(group_id);
    let (members, projects, mappings) = tokio::join!(
        repositories::groups::members::list_group_members(pool, group_id),
        list_linked_scopes(pool, &q),
        repositories::groups::mappings::list_group_ad_mappings(pool, group_id),
    );
    TabCounts {
        members: faults.take("member count", members.map(|m| m.len() as i64)),
        projects: faults.take("project count", projects.map(|p| p.len() as i64)),
        mappings: faults.take("mapping count", mappings.map(|m| m.len() as i64)),
    }
}

const fn member_query(group_id: &GroupId) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeTarget::Group(group_id),
        Attribution::Member,
        DEFAULT_WINDOW_DAYS,
    )
}

const fn exclusive_query(group_id: &GroupId) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeTarget::Group(group_id),
        Attribution::Exclusive,
        DEFAULT_WINDOW_DAYS,
    )
}

pub(super) async fn load_usage(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> ScopeUsageRow {
    let q = exclusive_query(group_id);
    faults.take("group usage", get_scope_usage(pool, &q).await)
}

pub(super) async fn load_usage_tab(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> UsageTabData {
    let q = member_query(group_id);
    let exclusive = exclusive_query(group_id);
    let (attribution, models, daily, skills, tools, members) = tokio::join!(
        list_scope_attribution_sources(pool, &exclusive),
        list_scope_top_models(pool, &q, LEADERBOARD_LIMIT),
        list_daily_requests(pool, &q),
        list_scope_top_skills(pool, &q, LEADERBOARD_LIMIT),
        list_scope_top_tools(pool, &q, LEADERBOARD_LIMIT),
        list_member_usage(pool, &q),
    );
    let mut leaderboard: Vec<MemberUsageRow> = faults
        .take("member leaderboard", members)
        .into_values()
        .collect();
    leaderboard.sort_by(|a, b| {
        b.cost_microdollars
            .cmp(&a.cost_microdollars)
            .then_with(|| b.requests.cmp(&a.requests))
    });
    leaderboard.truncate(LEADERBOARD_LIMIT as usize);
    let models = faults.take("model mix", models);
    let daily = faults
        .take("daily requests", daily)
        .into_iter()
        .map(|d| d.requests)
        .collect();
    let skills = faults
        .take("top skills", skills)
        .into_iter()
        .map(|s| (s.skill, s.invocations))
        .collect();
    let tools = faults
        .take("top tools", tools)
        .into_iter()
        .map(|t| {
            (
                format!("{} · {}", t.server_name, t.tool_name),
                t.invocations,
            )
        })
        .collect();
    UsageTabData {
        attribution: faults.take("attribution sources", attribution),
        models,
        daily,
        skills,
        tools,
        leaderboard,
    }
}

pub(super) async fn load_members(
    pool: &PgPool,
    group_id: &GroupId,
    can_manage: bool,
    faults: &mut Faults,
) -> MembersData {
    let rows = faults.take(
        "group members",
        repositories::groups::members::list_group_members(pool, group_id).await,
    );
    let q = member_query(group_id);
    let usage = faults.take("member usage", list_member_usage(pool, &q).await);
    let active = faults
        .take("enabled user ids", load_enabled_user_ids(pool).await)
        .into_iter()
        .collect();
    let addable = if can_manage {
        faults.take("addable users", load_addable_users(pool, &rows).await)
    } else {
        Vec::new()
    };
    MembersData {
        rows,
        usage,
        active,
        addable,
    }
}

async fn load_enabled_user_ids(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM users WHERE status = 'active'")
        .fetch_all(pool)
        .await
}

async fn load_addable_users(
    pool: &PgPool,
    members: &[GroupMemberRow],
) -> Result<Vec<UserOptionView>, sqlx::Error> {
    let existing: std::collections::HashSet<&str> =
        members.iter().map(|m| m.user_id.as_str()).collect();
    let rows = sqlx::query!(
        r#"SELECT id AS "id!",
                  COALESCE(display_name, full_name, name) AS "name?",
                  email AS "email!"
           FROM users
           WHERE NOT ('anonymous' = ANY(roles))
           ORDER BY COALESCE(display_name, full_name, name, id)"#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter(|row| !existing.contains(row.id.as_str()))
        .map(|row| UserOptionView {
            label: row
                .name
                .map_or_else(|| row.email.clone(), |n| format!("{n} ({})", row.email)),
            user_id: UserId::new(row.id),
        })
        .collect())
}

pub(super) async fn load_projects(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> Vec<LinkedScopeRow> {
    faults.take(
        "group projects",
        list_linked_scopes(pool, &member_query(group_id)).await,
    )
}

pub(super) async fn load_mappings(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> Vec<GroupAdMappingRow> {
    faults.take(
        "directory mappings",
        repositories::groups::mappings::list_group_ad_mappings(pool, group_id).await,
    )
}

pub(super) async fn load_access(
    pool: &PgPool,
    group_id: &GroupId,
    faults: &mut Faults,
) -> Vec<AccessSectionView> {
    let services_path = match systemprompt_web_admin_core::request::get_services_path() {
        Ok(path) => path,
        Err(error) => {
            faults.record("access matrix", &error);
            return Vec::new();
        },
    };
    let mut sections = crate::handlers::access_control::build_matrix_sections(&services_path);
    if !sections.iter().any(|(kind, _, _)| kind == "marketplace") {
        let rows: Vec<(String, String, Option<String>)> = load_marketplaces()
            .into_iter()
            .map(|m| (m.id.to_string(), m.name, Some(m.description)))
            .collect();
        sections.insert(
            0,
            ("marketplace".to_owned(), "Marketplaces".to_owned(), rows),
        );
    }

    let subject =
        systemprompt_web_admin_core::repositories::entitlement::group_subject(group_id.as_str());
    let resolved = faults.take(
        "access matrix",
        systemprompt_web_admin_core::repositories::entitlement::resolve_subject_matrix(
            pool, &subject, sections,
        )
        .await,
    );

    let own_rules = faults.take(
        "access rules",
        systemprompt_web_admin_core::repositories::entitlement::list_all_rules(pool).await,
    );

    super::view::access_sections(resolved, &own_rules, group_id)
}

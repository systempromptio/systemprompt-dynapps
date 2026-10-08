//! `/admin/projects/{id}`: the three tabs and everything each one reads.
//!
//! The header tiles are exclusive attribution, so a project's numbers are its
//! own slice of the instance. The members table is the one member-attributed
//! view on the page — a person on two projects appears under both — and it
//! says so above the table rather than leaving the reader to reconcile it.

use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use crate::repositories::people_usage::attribution::{
    AttributionSourceRow, list_scope_attribution_sources,
};
use crate::repositories::people_usage::breakdown::{
    LinkedScopeRow, ModelUsageRow, list_linked_scopes, list_scope_top_models,
};
use crate::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, DailyRequests, LEADERBOARD_LIMIT, MemberUsageRow, ScopeUsageRow,
    get_scope_usage, list_daily_requests, list_member_usage,
};
use crate::repositories::projects::activity::{
    ProjectSessionRow, SkillEffectivenessRow, ToolHealthRow, list_project_sessions,
    list_project_skill_effectiveness, list_project_tool_health,
};
use systemprompt_web_admin_core::repositories::scope::{Attribution, ScopeQuery, ScopeTarget};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_core::types::projects::{ProjectMemberRow, ProjectRow};
use systemprompt_web_shared::ProjectId;

use super::super::people_view::{MemberContext, MemberInput, chips, mapping_rows, member_rows};
use super::super::types::{
    MemberRowView, ProjectDetailPageData, ProjectMembersTabView, ProjectSettingsTabView,
    UserOptionView,
};
use super::WINDOW_LABEL;
use super::detail_view::{gated_rows, kpis, tabs, usage_tab};

const SECTION_LIMIT: i64 = 25;

pub(super) struct DetailData {
    pub(super) usage: ScopeUsageRow,
    pub(super) members: Vec<ProjectMemberRow>,
    pub(super) member_usage: HashMap<String, MemberUsageRow>,
    pub(super) active: HashSet<String>,
    pub(super) addable: Vec<UserOptionView>,
    pub(super) groups: Vec<LinkedScopeRow>,
    pub(super) tool_calls: i64,
    pub(super) tool_success: i64,
    pub(super) skills: Vec<SkillEffectivenessRow>,
}

pub(super) struct ProjectUsageData {
    pub(super) attribution: Vec<AttributionSourceRow>,
    pub(super) daily: Vec<DailyRequests>,
    pub(super) models: Vec<ModelUsageRow>,
    pub(super) tools: Vec<ToolHealthRow>,
    pub(super) sessions: Vec<ProjectSessionRow>,
}

const fn exclusive(project_id: &ProjectId) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeTarget::Project(project_id),
        Attribution::Exclusive,
        DEFAULT_WINDOW_DAYS,
    )
}

const fn member_view(project_id: &ProjectId) -> ScopeQuery<'_> {
    ScopeQuery::new(
        ScopeTarget::Project(project_id),
        Attribution::Member,
        DEFAULT_WINDOW_DAYS,
    )
}

pub(super) async fn load(
    pool: &PgPool,
    project_id: &ProjectId,
    user_ctx: &UserContext,
    faults: &mut Faults,
) -> DetailData {
    let q = exclusive(project_id);
    let members_q = member_view(project_id);
    let members = faults.take(
        "project members",
        repositories::projects::members::list_project_members(pool, project_id).await,
    );

    let (usage, member_usage, groups, tools, skills) = tokio::join!(
        get_scope_usage(pool, &q),
        list_member_usage(pool, &members_q),
        list_linked_scopes(pool, &members_q),
        list_project_tool_health(pool, &q, SECTION_LIMIT),
        list_project_skill_effectiveness(pool, &q, LEADERBOARD_LIMIT),
    );
    let tools = faults.take("tool health", tools);
    let active = faults
        .take("active user ids", load_active_user_ids(pool).await)
        .into_iter()
        .collect();
    let addable = if user_ctx.is_admin {
        faults.take("addable users", load_addable_users(pool, &members).await)
    } else {
        Vec::new()
    };

    DetailData {
        usage: faults.take("project usage", usage),
        member_usage: faults.take("member usage", member_usage),
        groups: faults.take("project groups", groups),
        skills: faults.take("skill effectiveness", skills),
        tool_calls: tools.iter().map(|t| t.calls).sum(),
        tool_success: tools.iter().map(|t| t.calls - t.failures).sum(),
        active,
        addable,
        members,
    }
}

pub(super) async fn load_usage(
    pool: &PgPool,
    project_id: &ProjectId,
    faults: &mut Faults,
) -> ProjectUsageData {
    let q = exclusive(project_id);
    let (attribution, daily, models, tools, sessions) = tokio::join!(
        list_scope_attribution_sources(pool, &q),
        list_daily_requests(pool, &q),
        list_scope_top_models(pool, &q, LEADERBOARD_LIMIT),
        list_project_tool_health(pool, &q, SECTION_LIMIT),
        list_project_sessions(pool, &q, SECTION_LIMIT),
    );
    ProjectUsageData {
        attribution: faults.take("attribution sources", attribution),
        daily: faults.take("daily requests", daily),
        models: faults.take("model mix", models),
        tools: faults.take("tool health", tools),
        sessions: faults.take("project sessions", sessions),
    }
}

pub(super) struct TabReads<'a> {
    pub(super) tab: &'a str,
    pub(super) usage: Option<&'a ProjectUsageData>,
    pub(super) mappings: Vec<systemprompt_web_admin_core::types::projects::ProjectAdMappingRow>,
    pub(super) rules: &'a [systemprompt_web_admin_core::types::access_control::AccessControlRule],
}

pub(super) fn page_data(
    project: &ProjectRow,
    data: &DetailData,
    user_ctx: &UserContext,
    reads: TabReads<'_>,
) -> ProjectDetailPageData {
    let TabReads {
        tab,
        usage,
        mappings,
        rules,
    } = reads;
    let rows = member_views(data, user_ctx);

    ProjectDetailPageData {
        page: "project-detail",
        title: project.name.clone(),
        breadcrumbs: super::breadcrumbs(&project.name),
        tabs: tabs(&project.id, tab),
        active_tab: tab.to_owned(),
        window_label: WINDOW_LABEL.to_owned(),
        kpis: kpis(data, rows.len() as i64),
        group_count: data.groups.len() as i64,
        groups_represented: chips(&data.groups),
        members: (tab == "members").then(|| ProjectMembersTabView {
            count: rows.len() as i64,
            rows,
            addable_users: data.addable.clone(),
        }),
        usage: usage.map(|u| usage_tab(u, &data.skills)),
        settings: (tab == "settings").then(|| ProjectSettingsTabView {
            name: project.name.clone(),
            description: project.description.clone().unwrap_or_default(),
            cost_centre: project.cost_centre.clone().unwrap_or_default(),
            practice: project.practice.clone().unwrap_or_default(),
            source: project.source.clone(),
            mapping_count: mappings.len() as i64,
            feeding_groups: mapping_rows(
                mappings.into_iter().map(|m| (m.ad_group, m.source)),
                user_ctx.is_platform_admin,
            ),
            gated_count: rules.len() as i64,
            gated_entities: gated_rows(rules),
            can_map: user_ctx.is_platform_admin,
            can_delete: user_ctx.is_admin,
        }),
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        description: project.description.clone(),
        can_manage: user_ctx.is_admin,
        export: super::export_view(&project.id),
    }
}

pub(super) fn member_views(data: &DetailData, user_ctx: &UserContext) -> Vec<MemberRowView> {
    let inputs: Vec<MemberInput<'_>> = data.members.iter().map(as_member_input).collect();
    member_rows(
        &inputs,
        &MemberContext {
            usage: &data.member_usage,
            active: &data.active,
            can_manage: user_ctx.is_admin,
        },
    )
}

fn as_member_input(row: &ProjectMemberRow) -> MemberInput<'_> {
    MemberInput {
        user_id: row.user_id.as_str(),
        display_name: row.display_name.as_deref(),
        email: row.email.as_deref(),
        sources: &row.sources,
        source_ad_groups: &row.source_ad_groups,
        valid_until: row.valid_until,
    }
}

async fn load_active_user_ids(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT id AS "id!" FROM users WHERE status = 'active'"#)
        .fetch_all(pool)
        .await
}

async fn load_addable_users(
    pool: &PgPool,
    members: &[ProjectMemberRow],
) -> Result<Vec<UserOptionView>, sqlx::Error> {
    let existing: HashSet<&str> = members.iter().map(|m| m.user_id.as_str()).collect();
    let rows = sqlx::query_as::<_, (String, Option<String>, String)>(
        "SELECT id, COALESCE(display_name, full_name, name), email
         FROM users
         WHERE NOT ('anonymous' = ANY(roles))
         ORDER BY COALESCE(display_name, full_name, name, id)",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter(|(id, _, _)| !existing.contains(id.as_str()))
        .map(|(id, name, email)| UserOptionView {
            label: name.map_or_else(|| email.clone(), |n| format!("{n} ({email})")),
            user_id: UserId::new(id),
        })
        .collect())
}

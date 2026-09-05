//! Shaping the project listing tiles and the detail page context.

use crate::types::UserContext;
use crate::types::access_control::{AccessControlRule, AccessDecision};
use crate::types::projects::{ProjectMemberRow, ProjectRow};

use super::super::people_view::{
    MemberContext, MemberInput, chips, mapping_rows, member_rows, model_rows, name_count_rows,
    stat_tiles,
};
use super::super::types::{AccessRowView, ProjectDetailPageData, ProjectRowView, StatTileView};
use super::data::ProjectDetailData;

pub(super) fn project_stats(rows: &[ProjectRowView]) -> Vec<StatTileView> {
    let members: i64 = rows.iter().map(|r| r.member_count).sum();
    let requests: i64 = rows.iter().map(|r| r.requests_30d).sum();
    vec![
        StatTileView {
            label: "Projects",
            value: rows.len().to_string(),
        },
        StatTileView {
            label: "Members",
            value: members.to_string(),
        },
        StatTileView {
            label: "Requests 30d",
            value: requests.to_string(),
        },
    ]
}

pub(super) fn detail_page(
    project: ProjectRow,
    loaded: ProjectDetailData,
    user_ctx: &UserContext,
) -> ProjectDetailPageData {
    let inputs: Vec<MemberInput<'_>> = loaded.members.iter().map(as_member_input).collect();
    let members = member_rows(
        &inputs,
        &MemberContext {
            usage: &loaded.member_usage,
            active: &loaded.active,
            can_manage: user_ctx.is_admin,
        },
    );
    let member_count = members.len() as i64;

    ProjectDetailPageData {
        page: "project-detail",
        title: project.name.clone(),
        crumbs: super::crumbs(&project.name),
        stats: stat_tiles(&loaded.usage, member_count),
        members,
        addable_users: loaded.addable,
        groups_represented: chips(&loaded.groups),
        models: model_rows(&loaded.models),
        skills: name_count_rows(&loaded.skills),
        gated_entities: gated_rows(&loaded.rules),
        mappings: mapping_rows(
            loaded.mappings.into_iter().map(|m| (m.ad_group, m.source)),
            user_ctx.is_platform_admin,
        ),
        project_id: project.id,
        project_name: project.name,
        description: project.description,
        not_found: false,
        can_manage: user_ctx.is_admin,
        can_map: user_ctx.is_platform_admin,
    }
}

fn as_member_input(row: &ProjectMemberRow) -> MemberInput<'_> {
    MemberInput {
        user_id: row.user_id.as_str(),
        display_name: row.display_name.as_deref(),
        email: row.email.as_deref(),
        sources: &row.sources,
        source_ad_groups: &row.source_ad_groups,
    }
}

// Why: The entities gated *to* this project — rules whose subject is the
// project itself, which is the only entitlement a project carries.
fn gated_rows(rules: &[AccessControlRule]) -> Vec<AccessRowView> {
    rules
        .iter()
        .map(|r| AccessRowView {
            entity_type: r.entity_type.clone(),
            entity_id: r.entity_id.clone(),
            entity_name: r.entity_id.clone(),
            effective: match r.access {
                AccessDecision::Allow => "allow".to_owned(),
                AccessDecision::Deny => "deny".to_owned(),
            },
            layer: "project".to_owned(),
            detail: format!("project:{} {}", r.rule_value, r.entity_type),
            state: match r.access {
                AccessDecision::Allow => "allow",
                AccessDecision::Deny => "deny",
            },
        })
        .collect()
}

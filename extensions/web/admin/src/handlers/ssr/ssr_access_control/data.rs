//! Data assembly for the access-control page.
//!
//! The left tree is groups → members, with the `unassigned` group carrying
//! everyone the directory has not placed anywhere. Membership comes from the
//! `user_groups` view via the groups repository, so a derived unassigned member
//! is listed exactly like a real one.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::handlers::access_control::build_matrix_sections;
use crate::repositories;
use crate::types::UserSummary;
use crate::util::scope::SubjectScope;

use super::view::{AcEntityCatalogue, AcEntityView, AcGroupView, AcProjectView, AcUserView};

const UNASSIGNED: &str = "unassigned";

fn user_view(user: &UserSummary) -> AcUserView {
    let id = user.user_id.as_str().to_owned();
    let email = user
        .email
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    AcUserView {
        display_name: user.display_name.clone().unwrap_or_else(|| id.clone()),
        email,
        roles: user.roles.clone(),
        is_active: user.is_active,
        id,
    }
}

pub(super) async fn load_users(pool: &PgPool) -> Vec<UserSummary> {
    repositories::users::queries::list_users(pool, &SubjectScope::All)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: user listing failed"))
        .unwrap_or_default()
}

// Why: Groups, each with the members the `user_groups` view places in it.
pub(super) async fn load_groups(pool: &PgPool, users: &[UserSummary]) -> Vec<AcGroupView> {
    let by_id: HashMap<&str, &UserSummary> =
        users.iter().map(|u| (u.user_id.as_str(), u)).collect();
    let rows = repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: group listing failed"))
        .unwrap_or_default();

    let mut out = Vec::with_capacity(rows.len());
    for group in rows {
        let members = repositories::groups::members::list_group_members(pool, &group.id)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, group = %group.id, "member listing failed"))
            .unwrap_or_default();
        let members: Vec<AcUserView> = members
            .iter()
            .filter_map(|m| by_id.get(m.user_id.as_str()).map(|u| user_view(u)))
            .collect();
        out.push(AcGroupView {
            id: group.id,
            name: group.name,
            is_system: group.is_system,
            user_count: members.len(),
            users: members,
        });
    }
    // Why: the unassigned group is the one an operator most often needs and the
    // only one that shrinks as work gets done, so it leads the tree rather than
    // sorting alphabetically into the middle of it.
    out.sort_by_key(|g| (g.id != UNASSIGNED, g.name.to_lowercase()));
    out
}

pub(super) async fn load_projects(pool: &PgPool) -> Vec<AcProjectView> {
    repositories::projects::crud::list_project_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: project listing failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|p| AcProjectView {
            id: p.id,
            name: p.name,
            member_count: p.member_count,
        })
        .collect()
}

fn entities(
    sections: &[repositories::users::access_control::SectionInput],
    entity_type: &str,
) -> Vec<AcEntityView> {
    sections
        .iter()
        .find(|(kind, _, _)| kind == entity_type)
        .map(|(_, _, rows)| {
            rows.iter()
                .map(|(id, label, _)| AcEntityView {
                    id: id.clone(),
                    label: label.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

// Why: The entity catalogue the editor writes rules against — the same section
// list the per-user matrix endpoint resolves, so a row an operator can toggle
// here is a row the matrix will report on.
pub(super) fn entity_catalogue(services_path: &std::path::Path) -> AcEntityCatalogue {
    let sections = build_matrix_sections(services_path);
    AcEntityCatalogue {
        gateway_routes: entities(&sections, "gateway_route"),
        mcp_servers: entities(&sections, "mcp_server"),
        plugins: entities(&sections, "plugin"),
        agents: entities(&sections, "agent"),
        marketplaces: entities(&sections, "marketplace"),
        skills: entities(&sections, "skill"),
    }
}

//! The group, project and role choices the detail page's Membership and
//! Identity tabs edit.
//!
//! The detail page's membership checkboxes carry the *source* of each
//! membership — a directory-owned one is re-projected at every sign-in, so
//! offering to uncheck it would be a control that silently undoes itself.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;
use systemprompt_web_admin_core::ssr::faults::Faults;

use super::super::types::{MembershipChoiceView, RoleChoiceView};

fn membership_source(sources: &[String]) -> &'static str {
    if sources.iter().any(|s| s == "adfs") {
        "adfs"
    } else if sources.iter().any(|s| s == "manual") {
        "manual"
    } else {
        "derived"
    }
}

pub(super) async fn group_choices(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> Vec<MembershipChoiceView> {
    let held = faults.take(
        "group memberships",
        systemprompt_web_admin_core::repositories::principal::group_membership::list_group_ids_for_user(pool, user_id).await,
    );
    let groups = faults.take(
        "groups",
        systemprompt_web_admin_core::repositories::scope::containers::list_group_options(pool)
            .await,
    );
    let mut out = Vec::new();
    for group in groups {
        let is_held = held.iter().any(|id| id == &group.id);
        let source = if is_held {
            let members = faults.take(
                "group members",
                repositories::groups::members::list_group_members(pool, &group.id).await,
            );
            members
                .iter()
                .find(|m| m.user_id == user_id.as_str())
                .map_or("derived", |m| membership_source(&m.sources))
        } else {
            "manual"
        };
        out.push(MembershipChoiceView {
            id: group.id.as_str().to_owned(),
            name: group.name,
            held: is_held,
            source,
            from_directory: is_held && source == "adfs",
        });
    }
    out
}

pub(super) async fn project_choices(
    pool: &PgPool,
    user_id: &UserId,
    faults: &mut Faults,
) -> Vec<MembershipChoiceView> {
    let held = faults.take(
        "project memberships",
        repositories::projects::members::list_project_ids_for_user(pool, user_id).await,
    );
    faults
        .take(
            "projects",
            systemprompt_web_admin_core::repositories::scope::containers::list_project_options(
                pool,
            )
            .await,
        )
        .into_iter()
        .map(|project| {
            let is_held = held.iter().any(|id| id == &project.id);
            MembershipChoiceView {
                id: project.id.as_str().to_owned(),
                name: project.name,
                held: is_held,
                source: "manual",
                from_directory: false,
            }
        })
        .collect()
}

pub(super) fn role_choices(held: &[String]) -> Vec<RoleChoiceView> {
    systemprompt_web_admin_core::types::Role::ALL
        .iter()
        .map(|role| RoleChoiceView {
            id: role.as_str().to_owned(),
            label: role.label().to_owned(),
            held: held.iter().any(|r| r == role.as_str()),
            platform_only: matches!(
                role,
                systemprompt_web_admin_core::types::Role::PlatformAdmin
            ),
        })
        .collect()
}

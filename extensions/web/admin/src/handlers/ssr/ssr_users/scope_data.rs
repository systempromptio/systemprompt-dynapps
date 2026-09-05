//! The scope material the roster and detail pages need beyond the base rows.
//!
//! Group and project membership is a second dimension over the roster: the
//! table buckets by group, but a person sits in any number of projects, so the
//! ids arrive here as a per-user pass rather than being folded into the pure
//! enrichment transform. The detail page's checkboxes come from the same place,
//! carrying the *source* of each membership — a directory-owned one is
//! re-projected at every sign-in, so offering to uncheck it would be a control
//! that silently undoes itself.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories;

use super::super::types::{GroupBucket, MembershipChoiceView, RoleChoiceView};

// Why: Group and project display names, keyed by the id the roster buckets and
// filters on.
pub(super) async fn load_scope_names(
    pool: &PgPool,
) -> (
    std::collections::BTreeMap<String, String>,
    std::collections::BTreeMap<String, String>,
) {
    let groups = repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roster: group names unavailable"))
        .unwrap_or_default()
        .into_iter()
        .map(|g| (g.id, g.name))
        .collect();
    let projects = repositories::projects::crud::list_projects(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roster: project names unavailable"))
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.id, p.name))
        .collect();
    (groups, projects)
}

// Why: Fill in each roster row's project memberships. Kept separate from
// `load_user_groups` because the roster buckets by group: projects are a
// second, overlapping dimension and a user sits in any number of them.
pub(super) async fn attach_project_ids(pool: &PgPool, buckets: &mut [GroupBucket]) {
    for bucket in buckets.iter_mut() {
        for user in &mut bucket.users {
            user.project_ids =
                repositories::projects::members::list_project_ids_for_user(pool, &user.user_id)
                    .await
                    .inspect_err(
                        |e| tracing::warn!(error = %e, "roster: project membership unavailable"),
                    )
                    .unwrap_or_default();
        }
    }
}

fn membership_source(sources: &[String]) -> &'static str {
    if sources.iter().any(|s| s == "adfs") {
        "adfs"
    } else if sources.iter().any(|s| s == "manual") {
        "manual"
    } else {
        "derived"
    }
}

// Why: Every group as a checkbox, with the source of the membership the user
// already holds. A directory-sourced membership is reported so the template
// can render it read-only.
pub(super) async fn group_choices(pool: &PgPool, user_id: &UserId) -> Vec<MembershipChoiceView> {
    let held = repositories::groups::members::list_group_ids_for_user(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: group ids unavailable"))
        .unwrap_or_default();
    let mut out = Vec::new();
    for group in repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: group listing failed"))
        .unwrap_or_default()
    {
        let is_held = held.iter().any(|id| id == &group.id);
        let source = if is_held {
            let members = repositories::groups::members::list_group_members(pool, &group.id)
                .await
                .unwrap_or_default();
            members
                .iter()
                .find(|m| m.user_id == user_id.as_str())
                .map_or("derived", |m| membership_source(&m.sources))
        } else {
            "manual"
        };
        out.push(MembershipChoiceView {
            id: group.id,
            name: group.name,
            held: is_held,
            source,
            from_directory: is_held && source == "adfs",
        });
    }
    out
}

pub(super) async fn project_choices(pool: &PgPool, user_id: &UserId) -> Vec<MembershipChoiceView> {
    let held = repositories::projects::members::list_project_ids_for_user(pool, user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: project ids unavailable"))
        .unwrap_or_default();
    repositories::projects::crud::list_projects(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "user detail: project listing failed"))
        .unwrap_or_default()
        .into_iter()
        .map(|project| {
            let is_held = held.iter().any(|id| id == &project.id);
            MembershipChoiceView {
                id: project.id,
                name: project.name,
                held: is_held,
                source: "manual",
                from_directory: false,
            }
        })
        .collect()
}

// Why: The five flat roles as checkboxes. `platform_admin` is marked rather
// than filtered here so the template can decide whether to render it at all —
// only a platform admin may see or grant it.
pub(super) fn role_choices(held: &[String]) -> Vec<RoleChoiceView> {
    crate::types::Role::ALL
        .iter()
        .map(|role| RoleChoiceView {
            id: role.as_str().to_owned(),
            label: role.label().to_owned(),
            held: held.iter().any(|r| r == role.as_str()),
            platform_only: matches!(role, crate::types::Role::PlatformAdmin),
        })
        .collect()
}

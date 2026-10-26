//! Assembling the group listing: the rows, their usage, and the marketplaces
//! each group reaches.
//!
//! Entitlement is resolved through the same resolver the enforcement point
//! uses, against a synthetic subject holding one group membership — so the
//! chips on this page say what a member would actually reach rather than what
//! a rule table happens to mention.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories::groups::usage::{GroupUsageRow, list_groups_with_usage};
use systemprompt_web_admin_core::catalog_index::marketplaces::load_marketplaces;
use systemprompt_web_admin_core::repositories::scope::membership::UNATTRIBUTED as UNATTRIBUTED_ROW;
use systemprompt_web_admin_core::ssr::faults::Faults;

use super::super::people_view::{share, short_model};
use super::super::types::{GroupRowView, MemberSetChipView, UnattributedRowView};
use super::UNASSIGNED_GROUP;

pub(super) struct Listing {
    pub rows: Vec<GroupRowView>,
    pub unattributed: UnattributedRowView,
    pub unkeyed_people: i64,
}

pub(super) async fn load_listing(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
    window_days: i32,
    faults: &mut Faults,
) -> Listing {
    let loaded: Vec<GroupUsageRow> = faults.take(
        "group usage",
        list_groups_with_usage(pool, window_days).await,
    );

    let (remainder, groups): (Vec<GroupUsageRow>, Vec<GroupUsageRow>) =
        loaded.into_iter().partition(|g| g.id == UNATTRIBUTED_ROW);
    let remainder = remainder.into_iter().next();

    let marketplaces = load_group_marketplaces(services, pool, &groups, faults).await;
    let unkeyed_people = faults.take("unkeyed people count", count_unkeyed_people(pool).await);
    let remainder_requests = remainder.as_ref().map_or(0, |r| r.requests);
    let instance_requests: i64 =
        groups.iter().map(|g| g.requests).sum::<i64>() + remainder_requests;

    let rows = groups
        .into_iter()
        .map(|g| GroupRowView {
            href: format!("/admin/groups/{}", g.id),
            is_unassigned: g.id == UNASSIGNED_GROUP,
            marketplaces_title: marketplaces.get(&g.id).map_or_else(String::new, |chips| {
                chips
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }),
            marketplace_count: marketplaces.get(&g.id).map_or(0, Vec::len) as i64,
            project_count: g.project_count,
            member_count: g.member_count,
            active_members_30d: g.active_members,
            requests: g.requests,
            tokens: g.tokens,
            cost_microdollars: g.cost_microdollars,
            top_model_short: g.top_model.as_deref().map(short_model),
            top_model: g.top_model,
            top_model_requests: g.top_model_requests,
            source_abbrev: source_abbrev(source_label(&g.source)),
            source_label: source_label(&g.source),
            source_display: source_display(source_label(&g.source)),
            source: g.source,
            id: g.id,
            name: g.name,
            description: g.description,
        })
        .collect();

    Listing {
        unattributed: UnattributedRowView {
            requests: remainder_requests,
            tokens: remainder.as_ref().map_or(0, |r| r.tokens),
            cost_microdollars: remainder.as_ref().map_or(0, |r| r.cost_microdollars),
            share_pct: share(remainder_requests, instance_requests),
            people: unkeyed_people,
        },
        unkeyed_people,
        rows,
    }
}

fn source_abbrev(label: &str) -> &'static str {
    match label {
        "Directory" => {
            "Members follow the mapped Active Directory group and are re-projected at every sign-in"
        },
        "System" => {
            "Built in: the people who are in no group at all, never written and never deleted"
        },
        _ => "Created on this page; members are assigned here",
    }
}

fn source_label(source: &str) -> &'static str {
    match source {
        "yaml" | "adfs" => "Directory",
        "system" => "System",
        _ => "Dashboard",
    }
}

fn source_display(label: &str) -> &'static str {
    match label {
        "Directory" => "Active Directory",
        "System" => "Built-in",
        _ => "Created here",
    }
}

async fn count_unkeyed_people(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "count!"
           FROM users u
           WHERE NOT ('anonymous' = ANY(u.roles))
             AND NOT EXISTS (
                 SELECT 1 FROM user_scope_defaults d
                 WHERE d.user_id = u.id AND d.primary_group_id IS NOT NULL)"#
    )
    .fetch_one(pool)
    .await
}

async fn load_group_marketplaces(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
    groups: &[GroupUsageRow],
    faults: &mut Faults,
) -> HashMap<String, Vec<MemberSetChipView>> {
    let section: Vec<(String, String, Option<String>)> = load_marketplaces()
        .into_iter()
        .map(|m| (m.id.to_string(), m.name, None))
        .collect();
    if section.is_empty() {
        return HashMap::new();
    }

    let subjects: Vec<_> = groups
        .iter()
        .map(|g| systemprompt_web_admin_core::repositories::entitlement::group_subject(&g.id))
        .collect();
    let sections = vec![("marketplace".to_owned(), "Marketplaces".to_owned(), section)];
    let matrices = faults.take(
        "group marketplaces",
        systemprompt_web_admin_core::repositories::entitlement::resolve_subject_matrices(
            pool, services, &subjects, &sections,
        )
        .await,
    );
    groups
        .iter()
        .zip(matrices)
        .map(|(group, matrix)| {
            let chips = matrix
                .into_iter()
                .flat_map(|s| s.rows)
                .filter(|r| r.effective == "allow")
                .map(|r| MemberSetChipView {
                    id: r.entity_id,
                    label: r.entity_name,
                })
                .collect();
            (group.id.clone(), chips)
        })
        .collect()
}

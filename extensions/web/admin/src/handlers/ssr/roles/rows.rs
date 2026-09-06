//! What the roles page renders: the role cards, the two tables' rows, the
//! sortable headers and the filter menus.
//!
//! Split from [`super::data`] along the line between deciding which rows
//! survive and deciding what they look like. Everything here reads a
//! [`RolesQuery`] and writes a view type; nothing here filters.

use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::roles::entitlements::RoleEntitlementRow;
use crate::repositories::roles::members::RoleMemberRow;
use crate::types::Role;
use crate::types::access_control::AccessDecision;

use super::data::RolesQuery;
use super::view::{RoleCardView, RoleEntitlementView, RoleMemberView, SelectOption};

fn label_for(role: &str) -> String {
    role.parse::<Role>()
        .map_or_else(|()| role.to_owned(), |r| r.label().to_owned())
}

fn tone_for(role: &str) -> &'static str {
    match role {
        "platform_admin" => "err",
        "admin" => "warn",
        "developer" | "project_manager" => "accent",
        _ => "ok",
    }
}

pub(super) fn cards(
    rows: &[RoleMemberRow],
    entitlements: &[RoleEntitlementRow],
    query: &RolesQuery,
) -> Vec<RoleCardView> {
    Role::ALL
        .iter()
        .map(|role| {
            let id = role.as_str();
            let held: Vec<&RoleMemberRow> = rows.iter().filter(|r| r.role == id).collect();
            let manual_count = held.iter().filter(|r| r.is_manual).count();
            let entitlement_count = entitlements.iter().filter(|e| e.role == id).count();
            let active = query.field("role") == Some(id);
            RoleCardView {
                id: id.to_owned(),
                label: role.label().to_owned(),
                member_count: held.len(),
                manual_count,
                directory_count: held.len() - manual_count,
                entitlement_count,
                tone: tone_for(id),
                href: if active {
                    query.url_with(&[("role", "")])
                } else {
                    query.url_with(&[("role", id)])
                },
                active,
                note: format!("{entitlement_count} entitlements · {manual_count} by hand"),
            }
        })
        .collect()
}

pub(super) fn member_views(rows: &[RoleMemberRow]) -> Vec<RoleMemberView> {
    rows.iter()
        .map(|row| {
            let id = row.user_id.as_str();
            RoleMemberView {
                display_name: row.display_name.clone().unwrap_or_else(|| id.to_owned()),
                email: row
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                role_label: label_for(&row.role),
                role: row.role.clone(),
                source: if row.is_manual { "Manual" } else { "Directory" },
                source_tone: if row.is_manual { "accent" } else { "muted" },
                is_manual: row.is_manual,
                is_active: row.is_active,
                status: if row.is_active { "Active" } else { "Inactive" },
                status_tone: if row.is_active { "ok" } else { "muted" },
                requests_30d: row.requests_30d,
                cost_30d_microdollars: row.cost_30d_microdollars,
                href: format!("/admin/users/{}", urlencoding::encode(id)),
                user_id: row.user_id.clone(),
            }
        })
        .collect()
}

pub(super) fn entitlement_views(rows: &[RoleEntitlementRow]) -> Vec<RoleEntitlementView> {
    rows.iter()
        .map(|row| RoleEntitlementView {
            role_label: label_for(&row.role),
            role: row.role.clone(),
            entity_type_label: row.entity_type.replace('_', " "),
            entity_type: row.entity_type.clone(),
            entity_id: row.entity_id.clone(),
            access: row.access.to_string(),
            access_tone: match row.access {
                AccessDecision::Allow => "ok",
                AccessDecision::Deny => "err",
            },
            default_included: row.default_included,
            default_label: if row.default_included {
                "Open by default"
            } else {
                "Closed by default"
            },
            href: format!(
                "/admin/access-control?entity_kind={}&subject_kind=role",
                row.entity_type
            ),
        })
        .collect()
}

pub(super) fn sort_headers(query: &RolesQuery) -> Vec<SortHeaderView> {
    let current = query.sort_key();
    let descending = query.descending();
    [
        (
            "person",
            "Person",
            "sp-col-person",
            "The account the role is held by.",
        ),
        (
            "role",
            "Role",
            "sp-col-role",
            "One row per role, so a person with three roles is three rows.",
        ),
        (
            "source",
            "Grant source",
            "sp-col-source",
            "Manual grants can be revoked here; directory grants are withdrawn in the directory.",
        ),
        (
            "requests",
            "Requests 30d",
            "sp-table__cell--num sp-col-num",
            "Gateway requests this account made in the last 30 days.",
        ),
        (
            "cost",
            "Cost 30d",
            "sp-table__cell--num sp-col-num",
            "What those requests cost.",
        ),
    ]
    .into_iter()
    .map(|(key, label, class, hint)| {
        let active = current == key;
        let next_dir = if active && !descending { "desc" } else { "" };
        SortHeaderView {
            label,
            class,
            hint,
            url: query.url_with(&[("sort", key), ("dir", next_dir)]),
            active,
            aria_sort: match (active, descending) {
                (true, false) => "ascending",
                (true, true) => "descending",
                (false, _) => "none",
            },
            indicator: match (active, descending) {
                (true, false) => "▲",
                (true, true) => "▼",
                (false, _) => "",
            },
        }
    })
    .collect()
}

pub(super) fn options(
    entries: &[(&str, &str)],
    all_label: &str,
    selected: Option<&str>,
) -> Vec<SelectOption> {
    let mut out = vec![SelectOption {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(entries.iter().map(|(value, label)| SelectOption {
        value: (*value).to_owned(),
        label: (*label).to_owned(),
        selected: selected == Some(*value),
    }));
    out
}

pub(super) fn role_options(query: &RolesQuery) -> Vec<SelectOption> {
    let entries: Vec<(&str, &str)> = Role::ALL.iter().map(|r| (r.as_str(), r.label())).collect();
    options(&entries, "All roles", query.field("role"))
}

pub(super) fn source_options(query: &RolesQuery) -> Vec<SelectOption> {
    options(
        &[
            ("manual", "Granted by hand"),
            ("directory", "From directory"),
        ],
        "Any grant source",
        query.field("source"),
    )
}

pub(super) fn status_options(query: &RolesQuery) -> Vec<SelectOption> {
    options(
        &[("active", "Active"), ("inactive", "Inactive")],
        "Any status",
        query.field("status"),
    )
}

//! Query parsing, filtering, sorting and pagination for `/admin/roles`.
//!
//! Every filter narrows the assignment rows in memory rather than in SQL. The
//! read is one pass over the (role, holder) pairs — bounded by the number of
//! accounts, not by traffic — and the page needs the unfiltered totals for
//! its role cards anyway, so a second filtered query would only be a second
//! chance for the two to disagree.

use serde::Deserialize;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::roles::entitlements::RoleEntitlementRow;
use crate::repositories::roles::members::RoleMemberRow;
use crate::types::Role;
use crate::types::access_control::AccessDecision;

use super::view::{RoleCardView, RoleEntitlementView, RoleMemberView, SelectOption};

pub(super) const BASE_URL: &str = "/admin/roles";
pub(super) const PAGE_SIZE: i64 = 50;
/// One row per role a person holds, so the ceiling is accounts × roles.
pub(super) const MEMBER_CAP: i64 = 5000;
pub(super) const ENTITLEMENT_CAP: i64 = 1000;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RolesQuery {
    pub role: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

impl RolesQuery {
    fn field(&self, name: &str) -> Option<&str> {
        let value = match name {
            "role" => self.role.as_deref(),
            "source" => self.source.as_deref(),
            "status" => self.status.as_deref(),
            "q" => self.q.as_deref(),
            _ => None,
        };
        value.filter(|v| !v.is_empty())
    }

    pub(super) fn any_applied(&self) -> bool {
        ["role", "source", "status", "q"]
            .iter()
            .any(|f| self.field(f).is_some())
    }

    fn sort_key(&self) -> &str {
        match self.sort.as_deref() {
            Some(key @ ("person" | "role" | "source" | "requests" | "cost")) => key,
            _ => "person",
        }
    }

    fn descending(&self) -> bool {
        self.dir.as_deref() == Some("desc")
    }

    // Why: every URL the page emits carries the filters that are set and
    // nothing else, so a link never resurrects a filter the operator cleared.
    fn url_with(&self, overrides: &[(&str, &str)]) -> String {
        let mut parts: Vec<(String, String)> = Vec::new();
        for name in ["role", "source", "status", "q", "sort", "dir"] {
            let overridden = overrides.iter().find(|(k, _)| *k == name);
            let value = match overridden {
                Some((_, v)) => (*v).to_owned(),
                None => match name {
                    "sort" => self.sort_key().to_owned(),
                    "dir" => {
                        if self.descending() {
                            "desc".to_owned()
                        } else {
                            String::new()
                        }
                    },
                    other => self.field(other).unwrap_or_default().to_owned(),
                },
            };
            if !value.is_empty() && !(name == "sort" && value == "person") {
                parts.push((name.to_owned(), value));
            }
        }
        for (key, value) in overrides {
            if *key == "page" && !value.is_empty() {
                parts.push(((*key).to_owned(), (*value).to_owned()));
            }
        }
        if parts.is_empty() {
            return BASE_URL.to_owned();
        }
        let query: Vec<String> = parts
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect();
        format!("{BASE_URL}?{}", query.join("&"))
    }
}

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
                sub: format!("{entitlement_count} entitlements · {manual_count} by hand"),
            }
        })
        .collect()
}

fn matches(row: &RoleMemberRow, query: &RolesQuery) -> bool {
    if let Some(role) = query.field("role")
        && row.role != role
    {
        return false;
    }
    if let Some(source) = query.field("source") {
        let manual = source == "manual";
        if row.is_manual != manual {
            return false;
        }
    }
    if let Some(status) = query.field("status") {
        let active = status == "active";
        if row.is_active != active {
            return false;
        }
    }
    if let Some(needle) = query.field("q") {
        let needle = needle.to_lowercase();
        let haystack = format!(
            "{} {} {}",
            row.user_id.as_str(),
            row.display_name.clone().unwrap_or_default(),
            row.email
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default()
        )
        .to_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }
    true
}

fn person_key(row: &RoleMemberRow) -> String {
    row.email
        .as_ref()
        .map(ToString::to_string)
        .or_else(|| row.display_name.clone())
        .unwrap_or_else(|| row.user_id.as_str().to_owned())
        .to_lowercase()
}

pub(super) fn filtered_sorted(rows: &[RoleMemberRow], query: &RolesQuery) -> Vec<RoleMemberRow> {
    let mut out: Vec<RoleMemberRow> = rows.iter().filter(|r| matches(r, query)).cloned().collect();
    match query.sort_key() {
        "role" => out.sort_by(|a, b| {
            a.role
                .cmp(&b.role)
                .then_with(|| person_key(a).cmp(&person_key(b)))
        }),
        "source" => out.sort_by_key(|r| (r.is_manual, person_key(r))),
        "requests" => out.sort_by_key(|r| (r.requests_30d, person_key(r))),
        "cost" => out.sort_by_key(|r| (r.cost_30d_microdollars, person_key(r))),
        _ => out.sort_by_key(person_key),
    }
    if query.descending() {
        out.reverse();
    }
    out
}

pub(super) fn member_views(rows: &[RoleMemberRow]) -> Vec<RoleMemberView> {
    rows.iter()
        .map(|row| {
            let id = row.user_id.as_str().to_owned();
            RoleMemberView {
                display_name: row.display_name.clone().unwrap_or_else(|| id.clone()),
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
                href: format!("/admin/users/{id}"),
                user_id: id,
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
        ("person", "Person", "", "The account the role is held by."),
        (
            "role",
            "Role",
            "",
            "One row per role, so a person with three roles is three rows.",
        ),
        (
            "source",
            "Grant source",
            "",
            "Manual grants can be revoked here; directory grants are withdrawn in the directory.",
        ),
        (
            "requests",
            "Requests 30d",
            "sp-table__cell--num",
            "Gateway requests this account made in the last 30 days.",
        ),
        (
            "cost",
            "Cost 30d",
            "sp-table__cell--num",
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

pub(super) fn search(query: &RolesQuery) -> String {
    query.field("q").unwrap_or_default().to_owned()
}

pub(super) fn page_index(query: &RolesQuery, total: i64) -> i64 {
    let last = (total.max(1) - 1) / PAGE_SIZE;
    query.page.unwrap_or(0).clamp(0, last)
}

pub(super) fn pagination(query: &RolesQuery, window: PageWindow) -> Pagination {
    let page = window.index;
    let prev_url = (page > 0).then(|| query.url_with(&[("page", &(page - 1).to_string())]));
    let next_url = (page + 1 < window.total_pages)
        .then(|| query.url_with(&[("page", &(page + 1).to_string())]));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

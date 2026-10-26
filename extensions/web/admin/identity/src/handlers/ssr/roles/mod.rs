//! `/admin/roles` — who holds each role, and what holding it opens.
//!
//! Two tabs, the role's side of two tables the rest of the console reads
//! from the user's side. *Members*: `user_manual_roles` says who was granted
//! what by hand, people listed once each with every role they hold.
//! *Entitlements*: `access_control_rules` at the `role` band says what a
//! grant reaches, grouped by role and entity kind. Reading is
//! the CONSOLE tier so a project manager can audit the estate; the grant and
//! revoke controls are MANAGE, and `platform_admin` narrows further to a
//! platform admin, which is the same rule `PUT /users/{id}/roles` enforces on
//! the write itself.

mod data;
mod entitlements_view;
mod rows;
mod view;

use std::sync::Arc;
use systemprompt_web_admin_core::ssr::page::Page;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::handlers::access_control::build_matrix_sections;
use crate::repositories::roles::{entitlements, members};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::get_route_labels;
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::list_view::PageWindow;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;
use systemprompt_web_admin_core::types::Role;
use systemprompt_web_admin_core::types::role::{ROLES_PLATFORM, has_any};

use data::RolesQuery;
use systemprompt_web_admin_core::ssr::list_view::DEFAULT_PAGE_SIZE;
use view::{HeaderFactView, RolesPageData};

const ENTITLEMENTS_URL: &str = "/admin/roles?tab=entitlements";

fn tabs(on_entitlements: bool, members: usize, entitlements: usize) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "members",
            label: "Members",
            href: data::BASE_URL.to_owned(),
            is_active: !on_entitlements,
            count: Some(i64::try_from(members).unwrap_or(i64::MAX)),
        },
        TabLinkView {
            slug: "entitlements",
            label: "Entitlements",
            href: ENTITLEMENTS_URL.to_owned(),
            is_active: on_entitlements,
            count: Some(i64::try_from(entitlements).unwrap_or(i64::MAX)),
        },
    ]
}

fn member_page(
    member_rows: &[members::RoleHolderRow],
    query: &RolesQuery,
) -> (Vec<members::RoleHolderRow>, i64, PageWindow) {
    let matching = data::filtered_sorted(member_rows, query);
    let total = i64::try_from(matching.len()).unwrap_or(i64::MAX);
    let index = data::page_index(query, total);
    let start = usize::try_from(index * DEFAULT_PAGE_SIZE).unwrap_or(0);
    let page_rows: Vec<_> = matching
        .iter()
        .skip(start)
        .take(usize::try_from(DEFAULT_PAGE_SIZE).unwrap_or(50))
        .cloned()
        .collect();
    let window = PageWindow::new(
        index,
        DEFAULT_PAGE_SIZE,
        total,
        i64::try_from(page_rows.len()).unwrap_or(0),
        "people",
    );
    (page_rows, total, window)
}

pub(crate) async fn roles_page(
    shell: Page,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<RolesQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = shell;
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let known: Vec<String> = Role::ALL.iter().map(|r| r.as_str().to_owned()).collect();
    let mut faults = Faults::new();
    let member_rows = faults.take(
        "role members",
        members::list_role_holders(&pool, &known, data::MEMBER_CAP).await,
    );
    let entitlement_rows = faults.take(
        "role entitlements",
        entitlements::list_role_entitlements(&pool, data::ENTITLEMENT_CAP).await,
    );

    let cards = rows::cards(&member_rows, &entitlement_rows, &query);
    let (page_rows, total, window) = member_page(&member_rows, &query);

    let on_entitlements = query.on_entitlements();
    let entitlement_groups =
        entitlement_groups(&services, on_entitlements, &entitlement_rows, &mut faults);

    let page = RolesPageData {
        page: "roles",
        title: "Roles & permissions",
        can_write: user_ctx.is_admin,
        can_grant_platform_admin: has_any(&user_ctx.roles, ROLES_PLATFORM),
        facts: facts(&cards, &entitlement_rows, &member_rows),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Roles & permissions"),
        ],
        tabs: tabs(on_entitlements, member_rows.len(), entitlement_rows.len()),
        on_entitlements,
        members: rows::member_views(&page_rows, &query),
        member_total: total,
        entitlement_total: entitlement_rows.len(),
        entitlement_groups,
        pagination: data::pagination(&query, window),
        sort_headers: rows::sort_headers(&query),
        role_options: rows::role_options(&query),
        source_options: rows::source_options(&query),
        status_options: rows::status_options(&query),
        search: data::search(&query),
        known_roles: rows::role_options(&RolesQuery::default()),
        filters_applied: query.any_applied(),
        clear_url: data::BASE_URL,
        cards,
    };

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &engine,
            "roles",
            &page,
            Viewer::new(&user_ctx, &mkt_ctx),
            &faults,
        ),
    )
}

fn entitlement_groups(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    on_entitlements: bool,
    entitlement_rows: &[entitlements::RoleEntitlementRow],
    faults: &mut Faults,
) -> Vec<entitlements_view::RoleEntitlementGroupView> {
    if !on_entitlements {
        return Vec::new();
    }
    let labels = faults.take("route labels", get_route_labels(&services.config()));
    let sections = faults.take(
        "services catalog",
        shared::get_services_path().map(|p| build_matrix_sections(services, &p)),
    );
    entitlements_view::groups(
        rows::entitlement_views(entitlement_rows, &labels),
        &sections,
    )
}

fn facts(
    cards: &[view::RoleCardView],
    entitlements: &[entitlements::RoleEntitlementRow],
    members: &[members::RoleHolderRow],
) -> Vec<HeaderFactView> {
    let manual: usize = members.iter().map(|m| m.manual_roles.len()).sum();
    let grants: usize = members.iter().map(|m| m.roles.len()).sum();
    let admins = cards
        .iter()
        .filter(|c| c.id == "admin" || c.id == "platform_admin")
        .map(|c| c.member_count)
        .sum::<usize>();
    let platform = cards
        .iter()
        .find(|c| c.id == "platform_admin")
        .map_or(0, |c| c.member_count);
    let denies = entitlements
        .iter()
        .filter(|e| {
            matches!(
                e.access,
                systemprompt_web_admin_core::types::access_control::AccessDecision::Deny
            )
        })
        .count();
    vec![
        HeaderFactView {
            value: members.len().to_string(),
            label: "people hold a role".to_owned(),
        },
        HeaderFactView {
            value: admins.to_string(),
            label: "on the admin plane".to_owned(),
        },
        HeaderFactView {
            value: platform.to_string(),
            label: if platform <= 1 {
                "platform admin, the last cannot be demoted".to_owned()
            } else {
                "platform admins".to_owned()
            },
        },
        HeaderFactView {
            value: manual.to_string(),
            label: format!("granted by hand, {} from the directory", grants - manual),
        },
        HeaderFactView {
            value: entitlements.len().to_string(),
            label: format!("role entitlements, {denies} of them denies"),
        },
    ]
}

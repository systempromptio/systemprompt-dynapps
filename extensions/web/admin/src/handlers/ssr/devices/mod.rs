//! `/admin/devices` — every credential the estate has handed out, and whether
//! anything is still using it.
//!
//! Four tabs over four tables: bridge sessions (a heartbeat), personal access
//! tokens (a secret), device certificates (a key), and enrolment links (a
//! promise not yet claimed). Each tab is listed as people — one row per
//! person, their machines or credentials folded beneath it — because a flat
//! listing repeats the same name once per laptop, restart and token, and the
//! fleet reads as six of one colleague. The KPI strip above the tabs is
//! fleet-wide and does not change with the tab, because the question that
//! brings anyone here — how much of this is still alive — is answered before
//! a tab is chosen.
//!
//! Every control is a link. Tab, filter, sort and page are all query
//! parameters, so a view is bookmarkable, the back button works, and the
//! server renders exactly one tab's rows rather than four. The links tab has
//! no filter chips: a pending link is waiting or expired, and both are listed.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::PageWindow;
use crate::handlers::ssr::types::{BreadcrumbView, FilterChipView};
use crate::repositories::config::security_policy::SecurityPolicy;
use crate::repositories::devices::pats::CredentialQuery;
use crate::repositories::devices::sessions::SessionQuery;
use crate::repositories::devices::stats::FleetStats;
use crate::repositories::devices::{sessions, stats};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod columns;
mod context;
mod credentials;
mod data;
mod tabs;
mod view;

use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;
use context::{DevicesPageContext, UserGroupView};
use tabs::{DeviceTab, resolve_sort};

pub(crate) const BASE_URL: &str = "/admin/devices";

#[derive(Debug, Deserialize)]
pub(crate) struct DevicesQuery {
    tab: Option<String>,
    state: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    stale: Option<String>,
    page: Option<i64>,
}

fn resolve_state(raw: Option<&str>) -> &'static str {
    match raw {
        Some("active") => "active",
        Some("revoked") => "revoked",
        _ => "all",
    }
}

async fn load_tab_rows(
    pool: &PgPool,
    tab: DeviceTab,
    query: &DevicesQuery,
    listing: Listing,
) -> (Vec<UserGroupView>, i64) {
    let Listing {
        state,
        sort,
        dir,
        offset,
        can_manage,
    } = listing;
    match tab {
        DeviceTab::Bridges => {
            data::load_sessions(
                pool,
                SessionQuery {
                    stale_only: query.stale.as_deref() == Some("1"),
                    sort,
                    dir,
                    limit: DEFAULT_PAGE_SIZE,
                    offset,
                },
            )
            .await
        },
        DeviceTab::Pats | DeviceTab::Certs => {
            let credentials = CredentialQuery {
                state,
                sort,
                dir,
                limit: DEFAULT_PAGE_SIZE,
                offset,
            };
            if tab == DeviceTab::Pats {
                credentials::load_pats(pool, credentials, can_manage).await
            } else {
                credentials::load_certs(pool, credentials, can_manage).await
            }
        },
        DeviceTab::Links => {
            credentials::load_links(pool, sort, dir, DEFAULT_PAGE_SIZE, offset).await
        },
    }
}

fn filter_chips(
    query: &DevicesQuery,
    tab: DeviceTab,
    state: &'static str,
    fleet: &FleetStats,
) -> Vec<FilterChipView> {
    match tab {
        DeviceTab::Bridges => view::build_stale_chips(query, fleet),
        DeviceTab::Links => Vec::new(),
        DeviceTab::Pats => {
            view::build_state_chips(query, state, (fleet.pats_total, fleet.pats_active))
        },
        DeviceTab::Certs => {
            view::build_state_chips(query, state, (fleet.certs_total, fleet.certs_active))
        },
    }
}

#[derive(Clone, Copy)]
struct Listing {
    state: &'static str,
    sort: &'static str,
    dir: &'static str,
    offset: i64,
    can_manage: bool,
}

pub(crate) async fn devices_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DevicesQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let tab = DeviceTab::parse(query.tab.as_deref());
    let sort = resolve_sort(tab, query.sort.as_deref());
    let dir = if query.dir.as_deref() == Some("asc") {
        "asc"
    } else {
        "desc"
    };
    let state = resolve_state(query.state.as_deref());
    let page = query.page.unwrap_or(0).max(0);
    let listing = Listing {
        state,
        sort,
        dir,
        offset: page * DEFAULT_PAGE_SIZE,
        can_manage: user_ctx.is_admin,
    };

    let fleet = stats::get_fleet_stats(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "device fleet stats failed"))
        .unwrap_or_default();
    let versions = sessions::list_version_counts(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge version histogram failed"))
        .unwrap_or_default();
    let (groups, users_total) = load_tab_rows(&pool, tab, &query, listing).await;

    let shown = groups.len();
    let window = PageWindow::new(
        page,
        DEFAULT_PAGE_SIZE,
        users_total,
        i64::try_from(shown).unwrap_or(DEFAULT_PAGE_SIZE),
        "people",
    );
    let version_bars = view::build_version_bars(&versions);

    let ctx = DevicesPageContext {
        page: "devices",
        title: "Devices & enrolments",
        breadcrumbs: vec![
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Devices & enrolments"),
        ],
        tabs: view::build_tabs(&query, tab, &fleet),
        tab: tab.slug(),
        stats: view::build_stats(&query, &fleet),
        has_versions: !version_bars.is_empty(),
        versions: version_bars,
        filters: filter_chips(&query, tab, state, &fleet),
        sort_headers: view::build_sort_headers(&query, tab.columns(), sort, dir),
        items_label: tab.items_label(),
        count_label: format!("{users_total} people"),
        has_rows: shown > 0,
        empty_message: tab.empty_message(),
        groups,
        pagination: view::build_pagination(&query, window),
        can_manage: listing.can_manage,
        has_row_actions: listing.can_manage && matches!(tab, DeviceTab::Pats | DeviceTab::Certs),
        max_pat_lifetime_days: SecurityPolicy::get().max_pat_lifetime_days,
    };

    Ok(super::render_typed_page(
        &engine, "devices", &ctx, &user_ctx, &mkt_ctx,
    ))
}

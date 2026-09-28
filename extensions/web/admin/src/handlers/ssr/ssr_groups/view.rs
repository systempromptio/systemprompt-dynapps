//! The group listing's page context, header tiles and page window.
//!
//! The tiles report the whole estate and are computed before the source
//! filter narrows the rows, so "Cost" reconciles with the instance total on
//! every filtered view rather than meaning something different on each.

use super::super::people_view::format_usd;
use super::super::types::{
    GroupKpiView, GroupRowView, GroupsPageData, MemberSetChipView, UnattributedRowView,
};
use super::data::Listing;
use super::sorting::BASE_URL;
use super::{UNASSIGNED_GROUP, sorting, tabs};
use systemprompt_web_admin_core::ssr::list_view::{
    DEFAULT_PAGE_SIZE, PageWindow, Pagination, paginate as build_pagination,
};
use systemprompt_web_admin_core::ssr::sync_plane::PlaneCardView;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

pub(super) fn kpis(
    rows: &[GroupRowView],
    unattributed: &UnattributedRowView,
    unkeyed_people: i64,
    range_label: &str,
) -> Vec<GroupKpiView> {
    let real: Vec<&GroupRowView> = rows.iter().filter(|r| !r.is_unassigned).collect();
    let members: i64 = real.iter().map(|r| r.member_count).sum();
    let unassigned = rows
        .iter()
        .find(|r| r.id == UNASSIGNED_GROUP)
        .map_or(0, |r| r.member_count);
    let requests: i64 = rows.iter().map(|r| r.requests).sum::<i64>() + unattributed.requests;
    let cost: i64 =
        rows.iter().map(|r| r.cost_microdollars).sum::<i64>() + unattributed.cost_microdollars;

    vec![
        GroupKpiView {
            label: "Groups",
            value: real.len().to_string(),
            note: format!("{unassigned} in no group"),
            tone: "",
        },
        GroupKpiView {
            label: "People in a group",
            value: members.to_string(),
            note: format!(
                "{} active",
                rows.iter().map(|r| r.active_members_30d).sum::<i64>()
            ),
            tone: "",
        },
        GroupKpiView {
            label: "Requests",
            value: requests.to_string(),
            note: format!("last {range_label}"),
            tone: "",
        },
        GroupKpiView {
            label: "Cost",
            value: format_usd(cost),
            note: "attributed at request time".to_owned(),
            tone: "accent",
        },
        GroupKpiView {
            label: "Unattributed",
            value: format!("{}%", unattributed.share_pct),
            note: format!("{} requests", unattributed.requests),
            tone: if unattributed.share_pct > 25 {
                "warn"
            } else {
                "ok"
            },
        },
        GroupKpiView {
            label: "Missing attribution keys",
            value: unkeyed_people.to_string(),
            note: if unkeyed_people == 0 {
                "all accounts keyed".to_owned()
            } else {
                "spend lands unattributed".to_owned()
            },
            tone: if unkeyed_people == 0 { "ok" } else { "warn" },
        },
    ]
}

#[derive(Clone, Copy)]
pub(super) struct ListingFacets<'a> {
    pub(super) range: &'a str,
    pub(super) sort: &'a str,
    pub(super) dir: &'a str,
    pub(super) source: &'a str,
}

pub(super) fn paginate(
    rows: Vec<GroupRowView>,
    page: i64,
    facets: ListingFacets<'_>,
) -> (Vec<GroupRowView>, Pagination) {
    let ListingFacets {
        range,
        sort,
        dir,
        source,
    } = facets;
    let total_rows = rows.len() as i64;
    let total_pages = page_count(total_rows);
    let current_page = page.clamp(1, total_pages);
    let start = (current_page - 1) * DEFAULT_PAGE_SIZE;
    let window: Vec<GroupRowView> = rows
        .into_iter()
        .skip(start.max(0) as usize)
        .take(DEFAULT_PAGE_SIZE as usize)
        .collect();
    let source_query = sorting::source_query(source);
    let url =
        |p: i64| format!("{BASE_URL}?range={range}&sort={sort}&dir={dir}{source_query}&page={p}");

    let zero_based_page = current_page - 1;
    let pagination = build_pagination(
        PageWindow::new(
            zero_based_page,
            DEFAULT_PAGE_SIZE,
            total_rows,
            window.len() as i64,
            "groups",
        ),
        |zero_based| url(zero_based + 1),
    );
    (window, pagination)
}

// Why: rustc's signed `i64::div_ceil` is unstable on the pinned toolchain, and
// clippy rejects the `(n + size - 1) / size` spelling in its favour.
fn page_count(total_rows: i64) -> i64 {
    (total_rows / DEFAULT_PAGE_SIZE + i64::from(total_rows % DEFAULT_PAGE_SIZE != 0)).max(1)
}


pub(super) struct GroupsPageInput<'a> {
    pub listing: Listing,
    pub window_days: i32,
    pub facets: ListingFacets<'a>,
    pub page: i64,
    pub sync: Option<PlaneCardView>,
    pub on_sync: bool,
    pub can_manage: bool,
    pub can_map: bool,
    pub group_options: Vec<MemberSetChipView>,
}

pub(super) fn page_data(input: GroupsPageInput<'_>) -> GroupsPageData {
    let GroupsPageInput {
        mut listing,
        window_days,
        facets,
        ..
    } = input;
    let range_label = sorting::range_label(window_days);
    let estate_kpis = kpis(
        &listing.rows,
        &listing.unattributed,
        listing.unkeyed_people,
        range_label,
    );
    if !facets.source.is_empty() {
        listing.rows.retain(|r| r.source_label == facets.source);
    }
    let total_groups = listing.rows.len() as i64;
    let (rows, pagination) = paginate(listing.rows, input.page, facets);
    GroupsPageData {
        export: systemprompt_web_admin_core::export::view::ExportView::single(
            "groups",
            &systemprompt_web_admin_core::export::view::query_string(&[(
                "days",
                Some(&window_days.to_string()),
            )]),
        ),
        page: "groups",
        title: "Groups",
        breadcrumbs: vec![BreadcrumbView::current("Groups")],
        tabs: tabs(input.on_sync),
        sync: input.sync,
        kpis: estate_kpis,
        sort_headers: sorting::sort_headers(facets.sort, facets.dir, facets.range, facets.source),
        ranges: sorting::range_links(window_days, facets.source),
        sources: sorting::source_links(facets.source, facets.range),
        source_filter: facets.source.to_owned(),
        range_label,
        toolbar_count: format!("{total_groups} groups · last {range_label}"),
        window_days,
        groups: rows,
        unattributed: listing.unattributed,
        pagination,
        total_groups,
        can_manage: input.can_manage,
        can_map: input.can_map,
        group_options: input.group_options,
        unkeyed_people: listing.unkeyed_people,
    }
}

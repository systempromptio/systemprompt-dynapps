//! Assembling the Skills page context: the scope strip, the tab strip, and
//! the active tab's body. The ribbon carries only the table's own filters;
//! the scope (window and marketplace) travels through it as preserved fields.

use chrono::Utc;

use super::adoption::adoption_view;
use super::rows::marketplace_name;
use super::tabs::{TabInputs, active_tab_body};
use super::views::{ScopeLinkView, SkillsPageContext};
use super::{SkillsQuery, SkillsTab};
use crate::handlers::ssr::analysis::help::skills_help;
use crate::handlers::ssr::analysis::reports::banner::ReportBannerView;
use crate::handlers::ssr::analysis::ribbon::{RibbonGroupView, RibbonView};
use crate::repositories::analysis::skills::{MarketplaceAdoptionRow, SkillFactRow, SkillSort};
use systemprompt_web_admin_core::catalog_index::inventory_index::{
    InventoryIndex, MarketplaceAudience,
};
use systemprompt_web_admin_core::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

pub(super) struct SkillsRead<'a> {
    pub(super) rows: &'a [SkillFactRow],
    pub(super) adoption: &'a [MarketplaceAdoptionRow],
    pub(super) audience: &'a MarketplaceAudience,
    pub(super) index: &'a InventoryIndex,
}

fn range_links(query: &SkillsQuery, tab: SkillsTab) -> Vec<TabLinkView> {
    [7, 30, 90, 365]
        .iter()
        .map(|d| TabLinkView {
            slug: match d {
                7 => "7",
                90 => "90",
                365 => "365",
                _ => "30",
            },
            label: match d {
                7 => "7 days",
                90 => "90 days",
                365 => "1 year",
                _ => "30 days",
            },
            href: query.link(tab, Some(*d), None),
            is_active: *d == query.days(),
            count: None,
        })
        .collect()
}

fn tab_links(query: &SkillsQuery, active: SkillsTab, row_count: usize) -> Vec<TabLinkView> {
    SkillsTab::ALL
        .iter()
        .map(|t| TabLinkView {
            slug: t.as_str(),
            label: t.label(),
            href: query.link(*t, None, None),
            is_active: *t == active,
            count: (*t == SkillsTab::Skills && *t != active)
                .then(|| i64::try_from(row_count).ok())
                .flatten(),
        })
        .collect()
}

fn marketplace_links(
    query: &SkillsQuery,
    tab: SkillsTab,
    index: &InventoryIndex,
) -> Vec<ScopeLinkView> {
    let selected = query.marketplace();
    let mut declared: Vec<(&str, &str)> = index
        .marketplaces
        .iter()
        .map(|m| (m.id.as_str(), m.name.as_str()))
        .collect();
    declared.sort_by_key(|(_, name)| *name);
    std::iter::once(ScopeLinkView {
        label: "All marketplaces".to_owned(),
        href: query.link_marketplace(tab, None),
        is_active: selected.is_none(),
    })
    .chain(declared.into_iter().map(|(id, name)| ScopeLinkView {
        label: name.to_owned(),
        href: query.link_marketplace(tab, Some(id)),
        is_active: selected.as_deref() == Some(id),
    }))
    .collect()
}

fn ribbon(query: &SkillsQuery, rows: &[SkillFactRow]) -> RibbonView {
    let clients: std::collections::BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.clients.iter().map(String::as_str))
        .collect();
    let client = SkillsQuery::trimmed(query.client.as_deref());
    let search = query.search();
    let sorted = (query.sort() != SkillSort::Invocations).then(|| query.sort().as_str());
    let mut preserved = vec![
        ("tab".to_owned(), SkillsTab::Skills.as_str().to_owned()),
        ("days".to_owned(), query.days().to_string()),
    ];
    if let Some(m) = query.marketplace() {
        preserved.push(("marketplace".to_owned(), m));
    }
    let mut ribbon = RibbonView::new(
        ANALYSIS_SKILLS_URL,
        query.link_without(SkillsTab::Skills, &["client", "search", "sort"]),
    )
    .preserve(&preserved)
    .group(RibbonGroupView::single(
        "client",
        "Client",
        "plug",
        client.as_deref(),
        clients.iter().map(|c| (*c, (*c).to_owned(), 0)),
    ))
    .group(RibbonGroupView::fixed(
        "sort",
        "Sort",
        "filter",
        sorted,
        &SkillSort::ALL.map(|(s, l)| (s.as_str(), l)),
    ))
    .search("search", search.as_deref(), "skill or plugin");
    if let Some(c) = client.as_deref() {
        ribbon = ribbon.chip(
            "Client",
            c,
            query.link_without(SkillsTab::Skills, &["client"]),
            Some(format!(
                "/admin/export/analysis-skills?{}&format=csv",
                export_query(query, SkillsTab::Skills)
            )),
        );
    }
    if let Some(q) = search.as_deref() {
        ribbon = ribbon.chip(
            "Search",
            q,
            query.link_without(SkillsTab::Skills, &["search"]),
            None,
        );
    }
    ribbon
}

fn export_query(query: &SkillsQuery, tab: SkillsTab) -> String {
    let table_filters_apply = tab == SkillsTab::Skills;
    let mut pairs = vec![("days", query.days().to_string())];
    for (key, value) in [
        ("marketplace", query.marketplace()),
        ("group", SkillsQuery::trimmed(query.group.as_deref())),
        ("project", SkillsQuery::trimmed(query.project.as_deref())),
        (
            "client",
            SkillsQuery::trimmed(query.client.as_deref()).filter(|_| table_filters_apply),
        ),
        ("search", query.search().filter(|_| table_filters_apply)),
    ] {
        if let Some(value) = value {
            pairs.push((key, value));
        }
    }
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) struct SkillsPageInputs {
    pub(super) is_admin: bool,
    pub(super) report_banner: ReportBannerView,
}

pub(super) fn build_context(
    query: &SkillsQuery,
    tab: SkillsTab,
    read: &SkillsRead<'_>,
    inputs: SkillsPageInputs,
) -> SkillsPageContext {
    let SkillsRead {
        rows,
        adoption,
        audience,
        index,
    } = *read;
    let marketplace = query.marketplace();
    let marketplace_name = marketplace.as_deref().map(|m| marketplace_name(index, m));
    let scoped_adoption: Vec<&MarketplaceAdoptionRow> = adoption
        .iter()
        .filter(|a| {
            marketplace
                .as_deref()
                .is_none_or(|m| m == a.marketplace_id.as_str())
        })
        .collect();
    let body = active_tab_body(&TabInputs {
        query,
        tab,
        read,
        scoped_adoption: &scoped_adoption,
        marketplace: marketplace.as_deref(),
        today: Utc::now().date_naive(),
    });
    let row_count = rows.len();
    SkillsPageContext {
        page: "analysis-skills",
        title: "Skills",
        tab: tab.as_str(),
        tabs: tab_links(query, tab, row_count),
        days: query.days(),
        range_links: range_links(query, tab),
        marketplace_links: marketplace_links(query, tab, index),
        scope_label: marketplace_name
            .clone()
            .unwrap_or_else(|| "All marketplaces".to_owned()),
        marketplace_name,
        ribbon: ribbon(query, rows),
        kpis: body.kpis,
        chart: body.chart,
        top_skills: body.top_skills,
        adoption: scoped_adoption
            .iter()
            .map(|a| adoption_view(query, a, audience, index))
            .collect(),
        groups: body.groups,
        row_count,
        unused_count: body.unused.len(),
        unused: body.unused,
        export: systemprompt_web_admin_core::export::view::ExportView::new(
            &["analysis-skills"],
            &export_query(query, tab),
        ),
        is_admin: inputs.is_admin,
        help: skills_help(),
        report_banner: inputs.report_banner,
        current_url: query.link(tab, None, None),
    }
}

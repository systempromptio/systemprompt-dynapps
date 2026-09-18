//! Assembling the Skills page context.

use chrono::Utc;

use super::SkillsQuery;
use super::adoption::adoption_view;
use super::rows::{group_rows, skill_row_view};
use super::summary::{chart, kpis, unused};
use super::views::{SkillFactView, SkillsPageContext};
use crate::handlers::ssr::analysis::help::skills_help;
use crate::handlers::ssr::analysis::reports::banner::ReportBannerView;
use crate::handlers::ssr::analysis::ribbon::{RibbonGroupView, RibbonView};
use crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use crate::handlers::ssr::types::TabLinkView;
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::{MarketplaceAdoptionRow, SkillFactRow, SkillSort};

// Why: what the page read, handed to the builder as one.
pub(super) struct SkillsRead<'a> {
    pub(super) rows: &'a [SkillFactRow],
    pub(super) adoption: &'a [MarketplaceAdoptionRow],
    pub(super) audience: &'a MarketplaceAudience,
    pub(super) index: &'a InventoryIndex,
}

fn range_links(query: &SkillsQuery) -> Vec<TabLinkView> {
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
            href: query.link(Some(*d), None),
            is_active: *d == query.days(),
            count: None,
        })
        .collect()
}

// Why: the ribbon's marketplace and client pills list what the instance
// declares and what the rows contain; sort is a pill too, so every control
// sits in one row.
fn ribbon(query: &SkillsQuery, rows: &[SkillFactRow], index: &InventoryIndex) -> RibbonView {
    let clients: std::collections::BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.clients.iter().map(String::as_str))
        .collect();
    let marketplace = SkillsQuery::trimmed(query.marketplace.as_deref());
    let client = SkillsQuery::trimmed(query.client.as_deref());
    let search = SkillsQuery::trimmed(query.search.as_deref());
    let sorted = (query.sort() != SkillSort::Invocations).then(|| query.sort().as_str());
    let mut ribbon = RibbonView::new(
        ANALYSIS_SKILLS_URL,
        query.link_without(&["marketplace", "client", "search", "sort"]),
    )
    .preserve(&[("days".to_owned(), query.days().to_string())])
    .group(RibbonGroupView::single(
        "marketplace",
        "Marketplace",
        "layers",
        marketplace.as_deref(),
        index
            .marketplaces
            .iter()
            .map(|m| (m.id.as_str(), m.name.clone(), 0)),
    ))
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
    let export = |name: &str, value: &str| {
        format!(
            "/admin/export/analysis-skills?days={}&{name}={}&format=csv",
            query.days(),
            urlencoding::encode(value)
        )
    };
    if let Some(m) = marketplace.as_deref() {
        let label = super::rows::marketplace_name(index, m);
        ribbon = ribbon.chip(
            "Marketplace",
            label,
            query.link_without(&["marketplace"]),
            Some(export("marketplace", m)),
        );
    }
    if let Some(c) = client.as_deref() {
        ribbon = ribbon.chip(
            "Client",
            c,
            query.link_without(&["client"]),
            Some(export("client", c)),
        );
    }
    if let Some(q) = search.as_deref() {
        ribbon = ribbon.chip("Search", q, query.link_without(&["search"]), None);
    }
    ribbon
}

// Why: what the handler read besides the rows.
pub(super) struct PageInputs {
    pub(super) is_admin: bool,
    pub(super) report_banner: ReportBannerView,
}

pub(super) fn build_context(
    query: &SkillsQuery,
    read: &SkillsRead<'_>,
    inputs: PageInputs,
) -> SkillsPageContext {
    let SkillsRead {
        rows,
        adoption,
        audience,
        index,
    } = *read;
    let today = Utc::now().date_naive();
    let views: Vec<SkillFactView> = rows
        .iter()
        .map(|r| skill_row_view(r, audience, index, today))
        .collect();
    let row_count = views.len();
    let unused = unused(rows, index, audience);
    SkillsPageContext {
        page: "analysis-skills",
        title: "Skills",
        days: query.days(),
        range_links: range_links(query),
        ribbon: ribbon(query, rows, index),
        kpis: kpis(rows, adoption, audience, today),
        chart: chart(rows, today),
        adoption: adoption
            .iter()
            .map(|a| adoption_view(a, audience, index))
            .collect(),
        groups: group_rows(views, index),
        row_count,
        unused_count: unused.len(),
        unused,
        export: crate::export::ExportView::new(
            &["analysis-skills"],
            &format!("days={}", query.days()),
        ),
        is_admin: inputs.is_admin,
        help: skills_help(),
        report_banner: inputs.report_banner,
        current_url: query.link(None, None),
    }
}

//! Assembling the Skills page context.

use chrono::Utc;

use super::SkillsQuery;
use super::adoption::adoption_view;
use super::rows::{group_rows, skill_row_view};
use super::summary::{chart, kpis, unused};
use super::views::{SkillFactView, SkillsPageContext};
use crate::handlers::ssr::list_view::SelectOptionView;
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

fn marketplace_options(query: &SkillsQuery, index: &InventoryIndex) -> Vec<SelectOptionView> {
    index
        .marketplaces
        .iter()
        .map(|m| SelectOptionView {
            selected: query.marketplace.as_deref() == Some(m.id.as_str()),
            value: m.id.as_str().to_owned(),
            label: m.name.clone(),
        })
        .collect()
}

fn client_options(query: &SkillsQuery, rows: &[SkillFactRow]) -> Vec<SelectOptionView> {
    let clients: std::collections::BTreeSet<&str> = rows
        .iter()
        .flat_map(|r| r.clients.iter().map(String::as_str))
        .collect();
    clients
        .iter()
        .map(|c| SelectOptionView {
            selected: query.client.as_deref() == Some(c),
            value: (*c).to_owned(),
            label: (*c).to_owned(),
        })
        .collect()
}

pub(super) fn build_context(
    query: &SkillsQuery,
    read: &SkillsRead<'_>,
    is_admin: bool,
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
        sort_options: SkillSort::ALL
            .iter()
            .map(|(s, label)| SelectOptionView {
                selected: *s == query.sort(),
                value: s.as_str().to_owned(),
                label: (*label).to_owned(),
            })
            .collect(),
        marketplace_options: marketplace_options(query, index),
        client_options: client_options(query, rows),
        search: SkillsQuery::trimmed(query.search.as_deref()).unwrap_or_default(),
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
        is_admin,
    }
}

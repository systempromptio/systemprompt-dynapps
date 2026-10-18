//! The template context for `/admin/governance`.
//!
//! Assembling it is a pure function of what the database returned and what the
//! URL asked for, so it is separated from the handler that does the I/O: the
//! shape below is the whole contract with `governance-warnings.hbs`.

use serde::Serialize;

use super::columns::columns;
use super::data::GovernanceData;
use super::filters::{decision_options, export_view, options, outcome_options, time_range_context};
use super::kpis::{GovernanceKpiView, StageFilterView, kpis, stage_filters};
use super::urls::{ColumnHeader, build_pagination, filter_url};
use super::{BASE_URL, GovernanceQuery, GovernanceTab, TabLink, view};
use crate::repositories::governance::decision_log::DecisionSort;
use systemprompt_web_admin_core::ssr::list_view::{
    DEFAULT_PAGE_SIZE, PageWindow, Pagination, ScopeFilterView, SelectOptionView, TimeRangeContext,
};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::util::time_range::TimeRange;

#[derive(Debug, Serialize)]
pub(super) struct RankView {
    name: String,
    detail: String,
    count: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct GovernancePageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) tabs: Vec<TabLink>,
    pub(super) tab_label: &'static str,
    pub(super) is_decisions: bool,
    pub(super) is_safety: bool,
    pub(super) is_hooks: bool,
    pub(super) time_range: TimeRangeContext,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) kpis: Vec<GovernanceKpiView>,
    pub(super) stage_filters: Vec<StageFilterView>,
    pub(super) attention: Vec<view::DecisionRow>,
    pub(super) attention_total: i64,
    pub(super) attention_more_url: String,
    pub(super) columns: Vec<ColumnHeader>,
    pub(super) decisions: Vec<view::DecisionRow>,
    pub(super) findings: Vec<view::FindingRow>,
    pub(super) hooks: Vec<view::HookRow>,
    pub(super) has_rows: bool,
    pub(super) row_count: String,
    pub(super) pagination: Pagination,
    pub(super) policies: Vec<SelectOptionView>,
    pub(super) categories: Vec<SelectOptionView>,
    pub(super) outcomes: Vec<SelectOptionView>,
    pub(super) search: String,
    pub(super) export: systemprompt_web_admin_core::export::view::ExportView,
    pub(super) clear_url: String,
    pub(super) has_filters: bool,
    pub(super) base_url: &'static str,
    pub(super) decision_options: Vec<SelectOptionView>,
    pub(super) pretool_24h: i64,
    pub(super) posttool_24h: i64,
    pub(super) top_policies: Vec<RankView>,
    pub(super) top_actors: Vec<RankView>,
}

pub(super) struct Build<'a> {
    pub(super) query: &'a GovernanceQuery,
    pub(super) tab: GovernanceTab,
    pub(super) range: TimeRange,
    pub(super) page: i64,
    pub(super) sort: DecisionSort,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) data: &'a GovernanceData,
}

fn has_filters(query: &GovernanceQuery) -> bool {
    query.policy.is_some()
        || query.decision.is_some()
        || query.category.is_some()
        || query.blocked.is_some()
        || query.q.as_deref().is_some_and(|q| !q.is_empty())
        || query.attention.as_deref().is_some_and(|a| !a.is_empty())
}

fn attention_more_url(query: &GovernanceQuery, data: &GovernanceData) -> String {
    if data.stats.attention_calls > i64::try_from(data.attention.len()).unwrap_or(0) {
        filter_url(query, &[("tab", "decisions"), ("attention", "1")])
    } else {
        String::new()
    }
}

fn top_policies(data: &GovernanceData) -> Vec<RankView> {
    data.top_policies
        .iter()
        .map(|p| RankView {
            name: p.policy.clone(),
            detail: p.tool_name.clone(),
            count: p.hits,
        })
        .collect()
}

fn top_actors(data: &GovernanceData) -> Vec<RankView> {
    data.top_actors
        .iter()
        .map(|a| RankView {
            name: a.display_name.clone(),
            detail: a.email.clone().unwrap_or_else(|| a.user_id.to_string()),
            count: a.deny_count,
        })
        .collect()
}

pub(super) fn build(input: Build<'_>) -> GovernancePageContext {
    let Build {
        query,
        tab,
        range,
        page,
        sort,
        scope_filter,
        data,
    } = input;

    let (rows_shown, total, noun) = match tab {
        GovernanceTab::Decisions => (data.decisions.len(), data.decision_total, "calls"),
        GovernanceTab::Safety => (data.findings.len(), data.finding_total, "findings"),
        GovernanceTab::Hooks => (
            data.hooks.len(),
            i64::try_from(data.hooks.len()).unwrap_or(0),
            "events",
        ),
    };
    let window = PageWindow::new(
        page,
        DEFAULT_PAGE_SIZE,
        total,
        i64::try_from(rows_shown).unwrap_or(0),
        noun,
    );

    GovernancePageContext {
        page: "governance-warnings",
        title: "Governance",
        subtitle: "The policy chain and the safety scanners, over one window.",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::current("Governance"),
        ],
        tabs: tab_links(query, tab, data),
        tab_label: tab.label(),
        is_decisions: tab == GovernanceTab::Decisions,
        is_safety: tab == GovernanceTab::Safety,
        is_hooks: tab == GovernanceTab::Hooks,
        time_range: time_range_context(query, range),
        scope_filter,
        kpis: kpis(query, data),
        stage_filters: stage_filters(query, data),
        attention: view::decision_rows(&data.attention),
        attention_total: data.stats.attention_calls,
        attention_more_url: attention_more_url(query, data),
        columns: columns(query, tab, sort),
        decisions: view::decision_rows(&data.decisions),
        findings: view::finding_rows(&data.findings),
        hooks: view::hook_rows(&data.hooks),
        has_rows: rows_shown > 0,
        row_count: format!("{total} {noun}"),
        pagination: build_pagination(query, window, noun),
        policies: options("All policies", query.policy.as_deref(), &data.policies),
        categories: options(
            "All categories",
            query.category.as_deref(),
            &data.categories,
        ),
        outcomes: outcome_options(query),
        search: query.q.clone().unwrap_or_default(),
        export: export_view(query, tab),
        clear_url: format!("{BASE_URL}?tab={}", tab.as_str()),
        has_filters: has_filters(query),
        base_url: BASE_URL,
        decision_options: decision_options(query),
        pretool_24h: data.pretool_24h,
        posttool_24h: data.posttool_24h,
        top_policies: top_policies(data),
        top_actors: top_actors(data),
    }
}

fn tab_links(
    query: &GovernanceQuery,
    active: GovernanceTab,
    data: &GovernanceData,
) -> Vec<TabLink> {
    [
        (GovernanceTab::Decisions, data.stats.evaluated),
        (GovernanceTab::Safety, data.safety.findings),
        (GovernanceTab::Hooks, data.pretool_24h + data.posttool_24h),
    ]
    .into_iter()
    .map(|(tab, count)| TabLink {
        label: tab.label(),
        href: filter_url(query, &[("tab", tab.as_str())]),
        is_active: tab == active,
        count,
    })
    .chain(
        std::iter::once(super::sync_tab::sync_link(false)).map(|t| TabLink {
            label: t.label,
            href: t.href,
            is_active: false,
            count: 0,
        }),
    )
    .collect()
}

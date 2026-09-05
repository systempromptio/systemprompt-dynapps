//! The template context for `/admin/governance`.
//!
//! Assembling it is a pure function of what the database returned and what the
//! URL asked for, so it is separated from the handler that does the I/O: the
//! shape below is the whole contract with `governance-warnings.hbs`.

use serde::Serialize;

use super::data::GovernanceData;
use super::urls::{ColumnHeader, build_pagination, filter_url, url_with};
use super::{BASE_URL, GovernanceQuery, GovernanceTab, TabLink, view};
use crate::handlers::ssr::list_view::{
    PageWindow, Pagination, ScopeFilterView, SelectOptionView, TimeRangeContext,
};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::decision_log::DecisionSort;
use crate::util::time_range::TimeRange;

/// One KPI tile, as `components/kpi` reads it.
#[derive(Debug, Serialize)]
pub(super) struct KpiView {
    label: &'static str,
    value: String,
    sub: String,
    tone: &'static str,
    href: String,
    active: bool,
    hint: &'static str,
}

/// One entry in a ranking list on the hooks tab.
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
    pub(super) kpis: Vec<KpiView>,
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
    pub(super) csv_url: String,
    pub(super) clear_url: String,
    pub(super) has_filters: bool,
    pub(super) base_url: &'static str,
    pub(super) decision_options: Vec<SelectOptionView>,
    pub(super) pretool_24h: i64,
    pub(super) posttool_24h: i64,
    pub(super) top_policies: Vec<RankView>,
    pub(super) top_actors: Vec<RankView>,
}

/// What one context build is derived from.
pub(super) struct Build<'a> {
    pub(super) query: &'a GovernanceQuery,
    pub(super) tab: GovernanceTab,
    pub(super) range: TimeRange,
    pub(super) page: i64,
    pub(super) sort: DecisionSort,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) data: &'a GovernanceData,
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
        GovernanceTab::Decisions => (data.decisions.len(), data.decision_total, "decisions"),
        GovernanceTab::Safety => (data.findings.len(), data.finding_total, "findings"),
        GovernanceTab::Hooks => (
            data.hooks.len(),
            i64::try_from(data.hooks.len()).unwrap_or(0),
            "events",
        ),
    };
    let window = PageWindow::new(
        page,
        super::PAGE_SIZE,
        total,
        i64::try_from(rows_shown).unwrap_or(0),
        noun,
    );

    GovernancePageContext {
        page: "governance-warnings",
        title: "Governance",
        subtitle: "Two enforcement planes over one window: the policy chain that \
                   decided each tool call, and the safety scanners that read every \
                   request and response.",
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
        csv_url: csv_url(query),
        clear_url: format!("{BASE_URL}?tab={}", tab.as_str()),
        has_filters: query.policy.is_some()
            || query.decision.is_some()
            || query.category.is_some()
            || query.blocked.is_some()
            || query.q.as_deref().is_some_and(|q| !q.is_empty()),
        base_url: BASE_URL,
        decision_options: decision_options(query),
        pretool_24h: data.pretool_24h,
        posttool_24h: data.posttool_24h,
        top_policies: data
            .top_policies
            .iter()
            .map(|p| RankView {
                name: p.policy.clone(),
                detail: p.tool_name.clone(),
                count: p.hits,
            })
            .collect(),
        top_actors: data
            .top_actors
            .iter()
            .map(|a| RankView {
                name: a.display_name.clone(),
                detail: a.email.clone().unwrap_or_else(|| a.user_id.to_string()),
                count: a.deny_count,
            })
            .collect(),
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
    .collect()
}

// Why: nine tiles, and the four in the middle are the four chain stages in
// evaluation order. Reading them left to right is reading the chain, which is
// the only ordering that lets an operator see which stage is doing the work.
fn kpis(query: &GovernanceQuery, data: &GovernanceData) -> Vec<KpiView> {
    let s = data.stats;
    let f = data.safety;
    let rate = percent(s.denied, s.evaluated);
    vec![
        tile(
            "Evaluated",
            s.evaluated,
            format!("{} callers", s.distinct_users),
            "",
            query,
            None,
            "Tool calls the chain decided in this window",
        ),
        tile(
            "Denied",
            s.denied,
            format!("{rate} of evaluated"),
            "err",
            query,
            Some(("decision", "deny")),
            "Calls the chain refused outright",
        ),
        tile(
            "Warned",
            s.warned,
            "would have been refused".to_owned(),
            "warn",
            query,
            Some(("decision", "warn")),
            "Warn mode absorbed these; enforcement would have refused them",
        ),
        tile(
            "Scope denies",
            s.scope_denied,
            "stage 1".to_owned(),
            "",
            query,
            Some(("policy", "agent_scope")),
            "Tool outside the scope granted to the agent",
        ),
        tile(
            "Secret denies",
            s.secret_denied,
            "stage 2".to_owned(),
            "",
            query,
            Some(("policy", "secret_scan")),
            "A credential pattern matched the tool input",
        ),
        tile(
            "Blocklist denies",
            s.blocklist_denied,
            "stage 3".to_owned(),
            "",
            query,
            Some(("policy", "tool_blocklist")),
            "Tool is blocked for this principal by policy",
        ),
        tile(
            "Rate denies",
            s.rate_denied,
            "stage 4".to_owned(),
            "",
            query,
            Some(("policy", "rate_limit")),
            "Call budget for the session was exhausted",
        ),
        tile(
            "Safety findings",
            f.findings,
            format!("{} categories", f.categories),
            "",
            query,
            Some(("tab", "safety")),
            "Scanner findings in both directions",
        ),
        tile(
            "Findings blocked",
            f.blocked,
            format!("{} audited only", f.audited),
            "err",
            query,
            Some(("tab", "safety")),
            "Findings with no blocks means the scanners are in warn mode",
        ),
    ]
}

fn tile(
    label: &'static str,
    value: i64,
    sub: String,
    tone: &'static str,
    query: &GovernanceQuery,
    filter: Option<(&'static str, &'static str)>,
    hint: &'static str,
) -> KpiView {
    let (href, active) = filter.map_or_else(
        || (filter_url(query, &[]), false),
        |(name, value)| {
            let current = match name {
                "decision" => query.decision.as_deref(),
                "policy" => query.policy.as_deref(),
                _ => query.tab.as_deref(),
            };
            (filter_url(query, &[(name, value)]), current == Some(value))
        },
    );
    KpiView {
        label,
        value: value.to_string(),
        sub,
        tone,
        href,
        active,
        hint,
    }
}

fn percent(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "0%".to_owned();
    }
    format!("{:.1}%", (part as f64 / whole as f64) * 100.0_f64)
}

fn columns(query: &GovernanceQuery, tab: GovernanceTab, sort: DecisionSort) -> Vec<ColumnHeader> {
    match tab {
        GovernanceTab::Decisions => vec![
            ColumnHeader::sortable(
                "When",
                "sp-col-date",
                "when",
                query,
                sort.key,
                sort.ascending,
            ),
            ColumnHeader::sortable(
                "Outcome",
                "sp-col-status",
                "decision",
                query,
                sort.key,
                sort.ascending,
            ),
            ColumnHeader::sortable(
                "Policy",
                "sp-col-text",
                "policy",
                query,
                sort.key,
                sort.ascending,
            ),
            ColumnHeader::plain("Stage", "sp-col-text"),
            ColumnHeader::sortable(
                "Tool",
                "sp-col-text",
                "tool",
                query,
                sort.key,
                sort.ascending,
            ),
            ColumnHeader::sortable(
                "User",
                "sp-col-identity",
                "user",
                query,
                sort.key,
                sort.ascending,
            ),
            ColumnHeader::plain("Scope", "sp-col-text"),
            ColumnHeader::plain("Reason", "sp-col-multiline"),
            ColumnHeader::plain("Trace", "sp-col-trace"),
        ],
        GovernanceTab::Safety => vec![
            ColumnHeader::plain("When", "sp-col-date"),
            ColumnHeader::plain("Category", "sp-col-text"),
            ColumnHeader::plain("Scanner", "sp-col-text"),
            ColumnHeader::plain("Severity", "sp-col-status"),
            ColumnHeader::plain("Direction", "sp-col-text"),
            ColumnHeader::plain("Outcome", "sp-col-status"),
            ColumnHeader::plain("Model", "sp-col-model"),
            ColumnHeader::plain("Excerpt", "sp-col-multiline"),
            ColumnHeader::plain("Request", "sp-col-trace"),
        ],
        GovernanceTab::Hooks => vec![
            ColumnHeader::plain("When", "sp-col-date"),
            ColumnHeader::plain("Hook", "sp-col-text"),
            ColumnHeader::plain("Tool", "sp-col-text"),
            ColumnHeader::plain("Plugin", "sp-col-text"),
            ColumnHeader::plain("User", "sp-col-identity"),
            ColumnHeader::plain("Status", "sp-col-status"),
        ],
    }
}

fn options(all_label: &str, selected: Option<&str>, values: &[String]) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.iter().map(|value| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: value.clone(),
    }));
    out
}

fn decision_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "deny", "warn", "allow"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.decision.as_deref().unwrap_or("") == value,
        })
        .collect()
}

fn outcome_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "blocked", "audited"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.blocked.as_deref().unwrap_or("") == value,
        })
        .collect()
}

fn time_range_context(query: &GovernanceQuery, range: TimeRange) -> TimeRangeContext {
    TimeRangeContext {
        preset: query.preset.clone().unwrap_or_else(|| "7d".to_owned()),
        from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
        to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
        base_url: BASE_URL,
        query: "",
    }
}

fn csv_url(query: &GovernanceQuery) -> String {
    url_with(query, &[("page", "")]).replacen(BASE_URL, &format!("{BASE_URL}/warnings.csv"), 1)
}

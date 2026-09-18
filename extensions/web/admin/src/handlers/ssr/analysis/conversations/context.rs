//! Assembling the conversations page context from the query, the scope and
//! the repository result: options, facets, breakdown rows, sort headers.

use crate::handlers::ssr::list_view::{PageWindow, ScopeFilterView, SelectOptionView};
use crate::repositories::analysis::conversations::{
    BreakdownBy, ConversationAnalysisResult, FlagFilter, JudgedFilter,
};
use crate::repositories::scope::ScopeRequest;
use systemprompt::identifiers::UserId;

use super::charts::series_charts;
use super::filters::{filter_view, sort_headers};
use super::kpis::KpisView;
use super::query::{ConversationAnalysisQuery, PAGE_SIZE};
use super::summary::{CATEGORIES, ConversationBucketView, OUTCOMES, category_label};
use super::view::{ConversationRowView, ConversationsPageContext, FacetLinkView};

// Why: this page's `since` labels reach past the live window contract (90d,
// all); the export offers the nearest window it can serve.
fn export_preset(params: &ConversationAnalysisQuery) -> &'static str {
    match params.since_label() {
        "24h" => "24h",
        "7d" => "7d",
        _ => "30d",
    }
}

fn options<'a, I>(selected: Option<&str>, items: I) -> Vec<SelectOptionView>
where
    I: Iterator<Item = (&'a str, String)>,
{
    items
        .map(|(value, label)| SelectOptionView {
            selected: selected == Some(value),
            value: value.to_owned(),
            label,
        })
        .collect()
}

fn breakdown_rows(
    params: &ConversationAnalysisQuery,
    data: &ConversationAnalysisResult,
) -> Vec<ConversationBucketView> {
    let by = params.breakdown();
    data.breakdown
        .iter()
        .map(|row| {
            let href = match by {
                BreakdownBy::Category => Some(params.narrowed("category", &row.label)),
                BreakdownBy::Outcome => Some(params.narrowed("outcome", &row.label)),
                BreakdownBy::Model => Some(params.narrowed("model", &row.label)),
                BreakdownBy::Client => Some(params.narrowed("client", &row.label)),
                BreakdownBy::Skill => Some(params.narrowed("skill", &row.label)),
                BreakdownBy::User => row
                    .user_id
                    .as_ref()
                    .map(|u| params.narrowed("user_id", u.as_str())),
                BreakdownBy::Group | BreakdownBy::Project => None,
            };
            let label = if by == BreakdownBy::Category {
                Some(category_label(&row.label))
            } else {
                None
            };
            ConversationBucketView::new(row, href, label, data.totals.total_cost_microdollars)
        })
        .collect()
}

fn facets<'a>(
    params: &ConversationAnalysisQuery,
    name: &str,
    active: Option<&str>,
    items: impl Iterator<Item = (&'a str, i64)>,
) -> Vec<FacetLinkView> {
    items
        .take(8)
        .map(|(value, count)| FacetLinkView {
            label: value.to_owned(),
            href: if active == Some(value) {
                params
                    .link_prefix(&[name, "page"])
                    .trim_end_matches(['?', '&'])
                    .to_owned()
            } else {
                params.narrowed(name, value)
            },
            count,
            is_active: active == Some(value),
        })
        .collect()
}

// Why: the select controls: fixed vocabularies plus the facets the filtered
// set actually contains, each with the current choice marked.
struct Selects {
    categories: Vec<SelectOptionView>,
    outcomes: Vec<SelectOptionView>,
    judged: Vec<SelectOptionView>,
    flags: Vec<SelectOptionView>,
    skills: Vec<SelectOptionView>,
    users: Vec<SelectOptionView>,
    models: Vec<SelectOptionView>,
    clients: Vec<SelectOptionView>,
}

fn selects(params: &ConversationAnalysisQuery, data: &ConversationAnalysisResult) -> Selects {
    let user_id = params.user_id();
    Selects {
        categories: options(
            params.category().as_deref(),
            CATEGORIES.iter().map(|(v, l)| (*v, (*l).to_owned())),
        ),
        outcomes: options(
            params.outcome().as_deref(),
            OUTCOMES.iter().map(|(v, l)| (*v, (*l).to_owned())),
        ),
        judged: options(
            params.judged().map(JudgedFilter::as_str),
            JudgedFilter::ALL
                .iter()
                .map(|(v, l)| (v.as_str(), (*l).to_owned())),
        ),
        flags: options(
            params.flag().map(FlagFilter::as_str),
            FlagFilter::ALL
                .iter()
                .map(|(v, l)| (v.as_str(), (*l).to_owned())),
        ),
        skills: options(
            params.skill().as_deref(),
            data.skills.iter().map(|s| {
                (
                    s.skill.as_str(),
                    format!("{} ({})", s.skill, s.conversations),
                )
            }),
        ),
        users: options(
            user_id.as_ref().map(UserId::as_str),
            data.users.iter().map(|u| {
                (
                    u.user_id.as_str(),
                    u.display_name
                        .clone()
                        .unwrap_or_else(|| u.user_id.as_str().to_owned()),
                )
            }),
        ),
        models: options(
            params.model().as_deref(),
            data.models.iter().map(|m| {
                (
                    m.model.as_str(),
                    format!("{} ({})", m.model, m.conversations),
                )
            }),
        ),
        clients: options(
            params.client().as_deref(),
            data.clients.iter().map(|c| {
                (
                    c.client_kind.as_str(),
                    format!("{} ({})", c.client_kind, c.conversations),
                )
            }),
        ),
    }
}

fn export_view(params: &ConversationAnalysisQuery) -> crate::export::ExportView {
    crate::export::ExportView::new(
        &["analysis-conversations", "analysis-conversation-breakdown"],
        &params
            .preserved(&["page", "since", "preset"])
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .chain(std::iter::once(format!("preset={}", export_preset(params))))
            .collect::<Vec<_>>()
            .join("&"),
    )
}

pub(super) fn build_context(
    params: &ConversationAnalysisQuery,
    request: &ScopeRequest,
    data: &ConversationAnalysisResult,
    scope_filter: ScopeFilterView,
) -> ConversationsPageContext {
    let rows: Vec<ConversationRowView> = data
        .rows
        .iter()
        .map(ConversationRowView::from_fact)
        .collect();
    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(
        params.page(),
        PAGE_SIZE,
        data.totals.conversations,
        shown,
        "conversations",
    );
    let hourly = params.since_label() == "24h";
    let selects = selects(params, data);
    ConversationsPageContext {
        page: "analysis-conversations",
        title: "Conversations",
        kpis: KpisView::new(&data.totals, &data.series),
        charts: series_charts(&data.series, hourly),
        range_links: params.range_links(),
        filter: filter_view(params, request),
        categories: selects.categories,
        outcomes: selects.outcomes,
        judged_options: selects.judged,
        flag_options: selects.flags,
        skills: selects.skills,
        users: selects.users,
        models: selects.models,
        clients: selects.clients,
        model_facets: facets(
            params,
            "model",
            params.model().as_deref(),
            data.models
                .iter()
                .map(|m| (m.model.as_str(), m.conversations)),
        ),
        client_facets: facets(
            params,
            "client",
            params.client().as_deref(),
            data.clients
                .iter()
                .map(|c| (c.client_kind.as_str(), c.conversations)),
        ),
        scope_filter,
        breakdown_tabs: params.breakdown_tabs(),
        breakdown_label: params.breakdown().label(),
        breakdown: breakdown_rows(params, data),
        has_rows: !rows.is_empty(),
        count_label: format!("{} conversations", data.totals.conversations),
        rows,
        pagination: params.pagination(window),
        sort_headers: sort_headers(params),
        export: export_view(params),
    }
}

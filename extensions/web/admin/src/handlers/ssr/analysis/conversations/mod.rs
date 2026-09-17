//! `/admin/analysis/conversations` — what the gateway's conversations are
//! for: purpose category, summary and outcome per conversation as the
//! `conversation_classifier` job judged them, with the skills used, the
//! person, their group and project, and the cost.
//!
//! One breakdown at a time (category, group, project, person or outcome)
//! over the same filtered set the table pages through, so the buckets add up
//! to the KPI tiles. Filters: window, category, outcome, skill, person, free
//! text and the shared `?group=&project=` scope.

mod query;
mod view;

use crate::handlers::ssr::analysis::evaluation::trigger::evaluation_enabled;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::{PageWindow, SelectOptionView, scope_filter_view};
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::conversations::{
    BreakdownBy, ConversationAnalysisFilter, ConversationAnalysisPage, ConversationAnalysisResult,
    load_conversation_analysis_page,
};
use crate::repositories::analytics::conversation_rows::ConversationSort;
use crate::repositories::scope::ScopeRequest;
use systemprompt::identifiers::UserId;

use query::{BASE_URL, ConversationAnalysisQuery, PAGE_SIZE};
use view::{
    CATEGORIES, ConversationAnalysisFilterView, ConversationBucketView, ConversationRowView,
    KpisView, OUTCOMES, PageContext, SortHeaders,
};

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<ConversationAnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let request = ScopeRequest::from_query(
        &shell.user,
        params.group.as_deref(),
        params.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let filter = ConversationAnalysisFilter {
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
        user_id: params.user_id(),
        category: params.category(),
        outcome: params.outcome(),
        skill: params.skill(),
        free_text: params.free_text(),
        since: params.since_datetime(),
        until: None,
    };
    let data = load_conversation_analysis_page(
        &pool,
        &filter,
        ConversationAnalysisPage {
            sort: params.sort(),
            descending: params.descending(),
            limit: PAGE_SIZE,
            offset: params.page() * PAGE_SIZE,
            breakdown: params.breakdown(),
        },
    )
    .await?;
    let scope_filter = scope_filter_view(
        &pool,
        &shell.user,
        &request,
        BASE_URL,
        params.preserved(&["group", "project", "page"]),
    )
    .await;
    let context = build_context(&params, &request, &data, scope_filter);
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-conversations",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

fn options<'a>(
    selected: Option<&str>,
    items: impl Iterator<Item = (&'a str, String)>,
) -> Vec<SelectOptionView> {
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
                BreakdownBy::User => row
                    .user_id
                    .as_ref()
                    .map(|u| params.narrowed("user_id", u.as_str())),
                BreakdownBy::Group | BreakdownBy::Project => None,
            };
            let label = if by == BreakdownBy::Category {
                Some(view::category_label(&row.label))
            } else {
                None
            };
            ConversationBucketView::new(row, href, label)
        })
        .collect()
}

fn filter_view(
    params: &ConversationAnalysisQuery,
    request: &ScopeRequest,
) -> ConversationAnalysisFilterView {
    ConversationAnalysisFilterView {
        since: params.since_label(),
        q: params.free_text().unwrap_or_default(),
        category: params.category().unwrap_or_default(),
        outcome: params.outcome().unwrap_or_default(),
        skill: params.skill().unwrap_or_default(),
        user_key: params
            .user_id()
            .map(|u| u.as_str().to_owned())
            .unwrap_or_default(),
        group: request.group.clone().unwrap_or_default(),
        project: request.project.clone().unwrap_or_default(),
        by: params.breakdown().as_str(),
        sort: params.sort().as_str(),
        dir: if params.descending() { "desc" } else { "asc" },
    }
}

fn sort_headers(params: &ConversationAnalysisQuery) -> SortHeaders {
    SortHeaders {
        activity: params.sort_header(
            ConversationSort::Activity,
            "Last activity",
            "sp-col-date",
            "Most recent request on the conversation",
        ),
        turns: params.sort_header(
            ConversationSort::Turns,
            "Turns",
            "sp-table__cell--num",
            "Human prompts answered",
        ),
        tokens: params.sort_header(
            ConversationSort::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input and output tokens",
        ),
        cost: params.sort_header(
            ConversationSort::Cost,
            "Cost",
            "sp-table__cell--num",
            "Priced spend on the conversation",
        ),
    }
}

fn build_context(
    params: &ConversationAnalysisQuery,
    request: &ScopeRequest,
    data: &ConversationAnalysisResult,
    scope_filter: crate::handlers::ssr::list_view::ScopeFilterView,
) -> PageContext {
    let rows: Vec<ConversationRowView> = data.rows.iter().map(ConversationRowView::new).collect();
    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(
        params.page(),
        PAGE_SIZE,
        data.totals.conversations,
        shown,
        "conversations",
    );
    let user_id = params.user_id();
    PageContext {
        page: "analysis-conversations",
        title: "Conversations",
        kpis: KpisView::new(&data.totals),
        filter: filter_view(params, request),
        categories: options(
            params.category().as_deref(),
            CATEGORIES.iter().map(|(v, l)| (*v, (*l).to_owned())),
        ),
        outcomes: options(
            params.outcome().as_deref(),
            OUTCOMES.iter().map(|(v, l)| (*v, (*l).to_owned())),
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
        scope_filter,
        breakdown_tabs: params.breakdown_tabs(),
        breakdown_label: params.breakdown().label(),
        breakdown: breakdown_rows(params, data),
        has_rows: !rows.is_empty(),
        can_evaluate: evaluation_enabled(),
        back_url: params
            .link_prefix(&[])
            .trim_end_matches(['?', '&'])
            .to_owned(),
        count_label: format!("{} conversations", data.totals.conversations),
        rows,
        pagination: params.pagination(window),
        sort_headers: sort_headers(params),
    }
}

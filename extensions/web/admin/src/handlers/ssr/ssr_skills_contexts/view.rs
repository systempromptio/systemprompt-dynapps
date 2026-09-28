//! Link building and row shaping for the conversations list: the query state
//! every sort link, page link and tab carries, the sortable headers, the
//! pagination footer, and the repository-row → template-row transforms.

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analytics::conversation_rows::ConversationSort;

use super::context::{ContextsSortHeaders, ModelOptionView, UserForFilterView};
use super::load::ContextsPageData;
use super::{BASE_URL, ContextsListQuery, ContextsPageInputs};
use crate::handlers::ssr::list_view::{
    DEFAULT_PAGE_SIZE, paginate_prefixed, query_string_dropping,
};

pub(super) fn preserved_query_string(query: &ContextsListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 11] = [
        ("group", query.group.as_deref()),
        ("project", query.project.as_deref()),
        ("user_id", query.user_id.as_ref().map(UserId::as_str)),
        ("model", query.model.as_deref()),
        ("q", query.q.as_deref()),
        ("since", query.since.as_deref()),
        ("preset", query.preset.as_deref()),
        ("view", query.view.as_deref()),
        ("side", query.side.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    query_string_dropping(&pairs, &[], drop)
}

pub(super) fn link_prefix(query: &ContextsListQuery, drop: &[&str]) -> String {
    let qs = preserved_query_string(query, drop);
    if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    }
}

pub(super) fn side_toggle_url(query: &ContextsListQuery, active: bool) -> String {
    let qs = preserved_query_string(query, &["side", "page"]);
    match (qs.is_empty(), active) {
        (true, true) => BASE_URL.to_owned(),
        (true, false) => format!("{BASE_URL}?side=1"),
        (false, true) => format!("{BASE_URL}?{qs}"),
        (false, false) => format!("{BASE_URL}?{qs}&side=1"),
    }
}

pub(super) fn build_pagination(query: &ContextsListQuery, window: PageWindow) -> Pagination {
    paginate_prefixed(window, &link_prefix(query, &["page"]))
}

pub(super) fn build_sort_headers(
    query: &ContextsListQuery,
    active_col: ConversationSort,
    is_desc: bool,
) -> ContextsSortHeaders {
    let prefix = link_prefix(query, &["sort", "dir", "page"]);
    let header = |col: ConversationSort, label: &'static str, class: &'static str, hint| {
        let active = col == active_col;
        let next_dir = SortHeaderView::next_dir(active, is_desc);
        SortHeaderView::new(
            (label, class, hint),
            format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            is_desc,
        )
    };
    ContextsSortHeaders {
        activity: header(
            ConversationSort::Activity,
            "Last activity",
            "sp-col-date",
            "Most recent request on the conversation",
        ),
        turns: header(
            ConversationSort::Turns,
            "Turns",
            "sp-table__cell--num",
            "Human prompts answered in this conversation",
        ),
        tokens: header(
            ConversationSort::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens across the conversation",
        ),
        cost: header(
            ConversationSort::Cost,
            "Cost",
            "sp-table__cell--num",
            "Billed cost across the conversation, side calls included",
        ),
    }
}

pub(super) fn page_offset(page: i64) -> i64 {
    page.max(0) * DEFAULT_PAGE_SIZE
}

pub(super) fn view_tabs(query: &ContextsListQuery, active: &str) -> Vec<TabLinkView> {
    let prefix = link_prefix(query, &["view", "page"]);
    [("users", "By user"), ("all", "All")]
        .into_iter()
        .map(|(slug, label)| TabLinkView {
            slug,
            label,
            href: format!("{prefix}view={slug}"),
            is_active: slug == active,
            count: None,
        })
        .collect()
}

pub(super) fn filter_options(
    inputs: &ContextsPageInputs,
    data: &ContextsPageData,
) -> ContextFilterOptions {
    ContextFilterOptions {
        users: data
            .users_for_filter
            .iter()
            .map(|u| UserForFilterView {
                selected: inputs.user_id.as_ref() == Some(&u.user_id),
                user_id: u.user_id.clone(),
                display_name: u.display_name.clone(),
            })
            .collect(),
        models: data
            .models
            .iter()
            .map(|m| ModelOptionView {
                selected: inputs.model.as_deref() == Some(m.as_str()),
                model: m.clone(),
            })
            .collect(),
    }
}

pub(super) struct ContextFilterOptions {
    pub(super) users: Vec<UserForFilterView>,
    pub(super) models: Vec<ModelOptionView>,
}

pub(super) fn preserved_filters(inputs: &ContextsPageInputs) -> Vec<(String, String)> {
    let side = if inputs.show_side { "1" } else { "" }.to_owned();
    vec![
        ("q".to_owned(), inputs.q.clone().unwrap_or_default()),
        ("model".to_owned(), inputs.model.clone().unwrap_or_default()),
        (
            "user_id".to_owned(),
            inputs
                .user_id
                .as_ref()
                .map(|u| u.as_str().to_owned())
                .unwrap_or_default(),
        ),
        (
            "since".to_owned(),
            inputs.since_label.clone().unwrap_or_default(),
        ),
        ("view".to_owned(), inputs.view.clone()),
        ("side".to_owned(), side),
    ]
}

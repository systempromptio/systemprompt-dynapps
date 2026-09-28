//! Link building for the history page: the query state every page and toggle
//! link carries.

use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::repositories::conversations::HistoryScope;
use systemprompt_web_admin_core::ssr::list_view::{PageWindow, Pagination};
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

use super::HistoryQuery;
use super::window::HistoryWindow;
use systemprompt_web_admin_core::ssr::list_view::paginate_prefixed;

pub(super) fn scope_label(scope: &HistoryScope) -> String {
    match scope {
        HistoryScope::All => "all users".to_owned(),
        HistoryScope::Users(_) => "your own conversations".to_owned(),
    }
}


fn query_parts(query: &HistoryQuery, keep_side: bool) -> Vec<String> {
    let mut parts = filter_parts(query, keep_side);
    parts.extend(HistoryWindow::of(query).query_parts());
    parts
}

fn filter_parts(query: &HistoryQuery, keep_side: bool) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(q) = query.q.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("q={}", urlencoding::encode(q)));
    }
    if let Some(u) = query
        .user_id
        .as_ref()
        .map(UserId::as_str)
        .filter(|s| !s.is_empty())
    {
        parts.push(format!("user_id={}", urlencoding::encode(u)));
    }
    if keep_side && query.show_side() {
        parts.push("side=1".to_owned());
    }
    parts
}

pub(super) fn side_toggle_url(query: &HistoryQuery, base: &str) -> String {
    let mut parts = query_parts(query, false);
    if !query.show_side() {
        parts.push("side=1".to_owned());
    }
    if parts.is_empty() {
        base.to_owned()
    } else {
        format!("{base}?{}", parts.join("&"))
    }
}

pub(super) fn build_pagination(query: &HistoryQuery, window: PageWindow, base: &str) -> Pagination {
    let parts = query_parts(query, true);
    let prefix = if parts.is_empty() {
        format!("{base}?")
    } else {
        format!("{base}?{}&", parts.join("&"))
    };
    paginate_prefixed(window, &prefix)
}

pub(super) fn window_links(query: &HistoryQuery, base: &str) -> Vec<TabLinkView> {
    let parts = filter_parts(query, true);
    let prefix = if parts.is_empty() {
        format!("{base}?")
    } else {
        format!("{base}?{}&", parts.join("&"))
    };
    HistoryWindow::of(query).tabs(&prefix)
}

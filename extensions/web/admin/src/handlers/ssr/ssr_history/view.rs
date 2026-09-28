//! Row shaping and link building for the history page: the title a row
//! shows, the detail page a viewer may open, and the query state every page
//! and toggle link carries.

use systemprompt::identifiers::UserId;

use crate::handlers::ssr::entity_urls::{context_detail_url, session_detail_url};
use crate::handlers::ssr::format::{
    attestation_title, client_chip, format_cost, format_token_total, relative_time,
};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::handlers::ssr::transcript_view::short_id;
use crate::handlers::ssr::types::TabLinkView;
use crate::repositories::analytics::conversations::{HistoryItem, HistoryScope, redact_text};
use crate::types::UserContext;

use super::context::HistoryRowView;
use super::window::HistoryWindow;
use super::{HistoryQuery, HistoryView};
use crate::handlers::ssr::list_view::paginate_prefixed;

pub(super) fn scope_label(scope: &HistoryScope) -> String {
    match scope {
        HistoryScope::All => "all users".to_owned(),
        HistoryScope::Users(_) => "your own conversations".to_owned(),
    }
}


pub(crate) fn detail_url(item: &HistoryItem, view: HistoryView) -> String {
    match view {
        HistoryView::Org => context_detail_url(&item.context_id),
        HistoryView::Own => format!(
            "/admin/history/conversations/{}",
            urlencoding::encode(item.context_id.as_str())
        ),
    }
}

pub(crate) fn session_url(item: &HistoryItem, viewer: &UserContext) -> Option<String> {
    viewer
        .is_admin
        .then(|| item.session_id.as_ref().map(session_detail_url))
        .flatten()
}

// Why: Claude Code sends a slash command's opening prompt as an XML
// `<command-message>` envelope, not typed words; the row is named by the
// command it ran instead.
pub fn command_name(prompt: &str) -> Option<String> {
    let open = "<command-name>";
    let close = "</command-name>";
    let start = prompt.find(open)? + open.len();
    let end = prompt[start..].find(close)? + start;
    let name = prompt[start..end].trim().trim_start_matches('/');
    (!name.is_empty()).then(|| format!("/{name}"))
}

fn conversation_title(item: &HistoryItem, short: &str) -> String {
    item.title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .or_else(|| {
            item.preview
                .as_deref()
                .map(|p| redact_text(p).0)
                .filter(|p| !p.trim().is_empty())
                .map(|p| command_name(&p).unwrap_or_else(|| p.chars().take(160).collect()))
        })
        .unwrap_or_else(|| format!("Conversation {short}"))
}

pub(crate) fn row_view(
    item: &HistoryItem,
    viewer: &UserContext,
    view: HistoryView,
) -> HistoryRowView {
    let when = item.last_at;
    let short = short_id(item.context_id.as_str());
    HistoryRowView {
        source: client_chip(&item.client_kind, &item.client_attestation),
        source_title: attestation_title(&item.client_attestation),
        conversation_title: conversation_title(item, &short),
        context_id: item.context_id.clone(),
        short_id: short,
        user_id: item.user_id.clone(),
        user_label: item.user_label.clone(),
        is_own: item.user_id == viewer.user_id,
        model: item.model.clone(),
        when_relative: relative_time(when),
        when_at: when.to_rfc3339(),
        entries_counted: item.turns,
        side_call_count: item.side_call_count,
        tokens_display: format_token_total(item.total_input_tokens + item.total_output_tokens),
        tokens_title: format!(
            "{} in / {} out",
            item.total_input_tokens, item.total_output_tokens
        ),
        cost_display: format_cost(item.cost_microdollars),
        detail_url: detail_url(item, view),
        session_url: session_url(item, viewer),
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

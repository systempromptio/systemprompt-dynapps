//! One conversation-history row, wherever a conversation is listed.
//!
//! `/admin/history` and `/admin/conversations` are one page at two scopes,
//! and a user's detail page lists the same rows; everything that differs
//! between the two listings is a method on [`HistoryView`], and [`row_view`]
//! shapes a [`HistoryItem`] the same way for all three.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};

use crate::repositories::conversations::{
    HistoryItem, HistoryScope, history_scope_for, own_history_scope, redact_text,
};
use crate::ssr::entity_urls::{context_detail_url, session_detail_url};
use crate::ssr::format::{
    ClientChipView, attestation_title, client_chip, format_cost, format_token_total, relative_time,
    short_id,
};
use crate::ssr::types::breadcrumb::BreadcrumbView;
use crate::types::UserContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryView {
    Own,
    Org,
}

impl HistoryView {
    pub const fn base_url(self) -> &'static str {
        match self {
            Self::Own => "/admin/history",
            Self::Org => "/admin/conversations",
        }
    }

    pub const fn page_id(self) -> &'static str {
        match self {
            Self::Own => "history",
            Self::Org => "conversations",
        }
    }

    pub const fn dataset(self) -> &'static str {
        match self {
            Self::Own => "history",
            Self::Org => "conversations",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Own => "My Conversations",
            Self::Org => "Conversations",
        }
    }

    pub const fn template(self) -> &'static str {
        match self {
            Self::Own => "history",
            Self::Org => "conversations",
        }
    }

    pub fn scope(self, ctx: &UserContext) -> HistoryScope {
        match self {
            Self::Own => own_history_scope(ctx),
            Self::Org => history_scope_for(ctx),
        }
    }

    pub fn breadcrumbs(self) -> Vec<BreadcrumbView> {
        match self {
            Self::Own => vec![
                BreadcrumbView::link("Account", "/admin/profile"),
                BreadcrumbView::current("My conversations"),
            ],
            Self::Org => vec![BreadcrumbView::current("Conversations")],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HistoryRowView {
    pub source: ClientChipView,
    pub source_title: String,
    pub short_id: String,
    pub context_id: ContextId,
    // Why: a Handlebars partial hash parameter shadows a same-named field at
    // every depth, so this row field keeps a name no hash will reuse.
    pub conversation_title: String,
    pub user_id: UserId,
    pub user_label: String,
    pub is_own: bool,
    pub model: Option<String>,
    pub when_relative: String,
    pub when_at: String,
    pub entries_counted: i64,
    pub side_call_count: i64,
    pub tokens_display: String,
    pub tokens_title: String,
    pub cost_display: String,
    pub detail_url: String,
    pub session_url: Option<String>,
}

pub fn detail_url(item: &HistoryItem, view: HistoryView) -> String {
    match view {
        HistoryView::Org => context_detail_url(&item.context_id),
        HistoryView::Own => format!(
            "/admin/history/conversations/{}",
            urlencoding::encode(item.context_id.as_str())
        ),
    }
}

pub fn session_url(item: &HistoryItem, viewer: &UserContext) -> Option<String> {
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

pub fn row_view(item: &HistoryItem, viewer: &UserContext, view: HistoryView) -> HistoryRowView {
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

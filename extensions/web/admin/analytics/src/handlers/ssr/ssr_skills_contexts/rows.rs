//! Repository-row → template-row transforms for the activity-by-person page:
//! one conversation as a table row, the rows grouped per person, and the
//! per-person summary with its folded model chips.

use std::collections::HashMap;

use systemprompt::identifiers::UserId;
use urlencoding::encode as urlencode;

use systemprompt_web_admin_core::repositories::conversations::rows::{
    ConversationRow, UserConversationSummary,
};
use systemprompt_web_admin_core::ssr::entity_urls::context_detail_url;
use systemprompt_web_admin_core::ssr::format::{
    client_chip, format_cost, format_token_total, relative_time, short_id,
};

use super::ContextsListQuery;
use super::context::{ConversationItemView, ModelChipView, UserSummaryView};
use super::view::link_prefix;

const PRIMARY_MODEL_CHIPS: usize = 2;


fn user_url(user_id: &UserId) -> String {
    format!("/admin/users/{}", urlencode(user_id.as_str()))
}

pub(super) fn conversation_item(c: &ConversationRow) -> ConversationItemView {
    let user_label = c
        .display_name
        .clone()
        .or_else(|| c.user_id.as_ref().map(|u| short_id(u.as_str())))
        .unwrap_or_else(|| "—".to_owned());
    ConversationItemView {
        context_id: c.context_id.clone(),
        detail_url: context_detail_url(&c.context_id),
        conversation_title: c.title.clone(),
        user_id: c.user_id.clone(),
        user_label,
        user_url: c.user_id.as_ref().map(user_url),
        model: c.model.clone(),
        client: client_chip(&c.client_kind, &c.client_attestation),
        turn_count: c.turn_count,
        side_call_count: c.side_call_count,
        tool_call_count: c.tool_call_count,
        error_count: c.error_count,
        tokens_display: format_token_total(c.total_input_tokens + c.total_output_tokens),
        tokens_title: format!(
            "{} in / {} out",
            c.total_input_tokens, c.total_output_tokens
        ),
        cost_display: format_cost(c.total_cost_microdollars),
        last_at: c.last_at.map(|t| t.to_rfc3339()),
        last_relative: c.last_at.map(relative_time),
    }
}

pub(super) fn group_by_user(
    rows: &[ConversationRow],
) -> HashMap<String, Vec<ConversationItemView>> {
    let mut out: HashMap<String, Vec<ConversationItemView>> = HashMap::new();
    for c in rows {
        let Some(user) = c.user_id.as_ref() else {
            continue;
        };
        out.entry(user.as_str().to_owned())
            .or_default()
            .push(conversation_item(c));
    }
    out
}

pub(super) fn user_summary(
    s: &UserConversationSummary,
    by_user: &HashMap<String, Vec<ConversationItemView>>,
    params: &ContextsListQuery,
) -> UserSummaryView {
    let conversations = by_user.get(s.user_id.as_str()).cloned().unwrap_or_default();
    UserSummaryView {
        user_id: s.user_id.clone(),
        user_label: s
            .display_name
            .clone()
            .unwrap_or_else(|| s.user_id.as_str().to_owned()),
        user_url: user_url(&s.user_id),
        conversation_count: s.conversation_count,
        tokens_display: format_token_total(s.total_tokens),
        all_conversations_url: format!(
            "{}view=all&user_id={}",
            link_prefix(params, &["view", "user_id", "page"]),
            urlencode(s.user_id.as_str())
        ),
        turn_count: s.turn_count,
        side_call_count: s.side_call_count,
        cost_display: format_cost(s.total_cost_microdollars),
        last_at: s.last_at.map(|t| t.to_rfc3339()),
        last_relative: s.last_at.map(relative_time),
        latest: s.latest.as_ref().map(conversation_item),
        models_primary: s
            .models
            .iter()
            .take(PRIMARY_MODEL_CHIPS)
            .map(|m| ModelChipView {
                model: m.model.clone(),
                requests: m.requests,
            })
            .collect(),
        models_more: s.models.len().saturating_sub(PRIMARY_MODEL_CHIPS),
        models_more_list: s
            .models
            .iter()
            .skip(PRIMARY_MODEL_CHIPS)
            .map(|m| format!("{} ({})", m.model, m.requests))
            .collect::<Vec<_>>()
            .join(", "),
        clients: s
            .clients
            .iter()
            .map(|c| client_chip(&c.client, &c.attestation))
            .collect(),
        has_conversations: !conversations.is_empty(),
        conversations,
    }
}

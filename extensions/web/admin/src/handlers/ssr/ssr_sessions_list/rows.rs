//! One repository row → one table row.

use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::format::{
    client_chip, format_cost, format_span, format_token_total, relative_time, short_id,
};
use crate::repositories::analytics::conversation_rows::ConversationRow;
use crate::repositories::session_state::session_state;

use super::context::SessionRowView;

const UNATTRIBUTED: &str = "unattributed";

pub(super) fn session_row(c: &ConversationRow) -> SessionRowView {
    let user_label = c
        .display_name
        .clone()
        .or_else(|| c.user_id.as_ref().map(|u| short_id(u.as_str())))
        .unwrap_or_else(|| "—".to_owned());

    SessionRowView {
        context_id: c.context_id.clone(),
        detail_url: context_detail_url(&c.context_id),
        conversation_title: c.title.clone(),
        session_id: c.session_id.clone(),
        user_id: c.user_id.clone(),
        user_label,
        user_url: c
            .user_id
            .as_ref()
            .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
        group_label: c
            .group_name
            .clone()
            .unwrap_or_else(|| UNATTRIBUTED.to_owned()),
        project_label: c
            .project_name
            .clone()
            .unwrap_or_else(|| UNATTRIBUTED.to_owned()),
        model: c.model.clone(),
        client: client_chip(&c.client_kind, &c.client_attestation),
        turn_count: c.turn_count,
        side_call_count: c.side_call_count,
        tool_call_count: c.tool_call_count,
        tokens_display: format_token_total(c.total_input_tokens + c.total_output_tokens),
        tokens_title: format!(
            "{} in / {} out",
            c.total_input_tokens, c.total_output_tokens
        ),
        cost_display: format_cost(c.total_cost_microdollars),
        duration_display: format_span(c.first_at, c.last_at),
        started_at: c.first_at.map(|t| t.to_rfc3339()),
        started_relative: c.first_at.map(relative_time),
        error_count: c.error_count,
        status_label: status_label(c),
        status_tone: status_tone(c),
    }
}

fn status_label(c: &ConversationRow) -> String {
    if c.error_count > 0 {
        let noun = if c.error_count == 1 {
            "error"
        } else {
            "errors"
        };
        return format!("{} {noun}", c.error_count);
    }
    state(c).label().to_owned()
}

fn status_tone(c: &ConversationRow) -> &'static str {
    if c.error_count > 0 {
        return "err";
    }
    state(c).tone()
}

fn state(c: &ConversationRow) -> crate::repositories::session_state::SessionState {
    let completed = c.status.as_deref() == Some("completed");
    session_state(chrono::Utc::now(), completed, c.last_at)
}

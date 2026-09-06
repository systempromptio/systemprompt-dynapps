//! View-model assembly for the user roster and detail pages.
//!
//! Pure transforms from repository rows into the serde structs the detail
//! page's Usage tab renders. The roster's own transforms live beside it in
//! `roster/view.rs`; what is left here is the gateway-usage shaping the tab
//! inherited from the retired per-user analytics page.

use super::super::types::UserConversationView;
use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::transcript_view::short_id;
use crate::repositories::analytics::contexts_list::ContextListItem;

fn local_stamp(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

// Why: the title falls back to the opening user turn and then to the short id —
// a gateway conversation has no name unless `ensure_context` managed to write
// one, and core swallows that failure, so an unnamed row is the common case.
pub(super) fn conversation_view(item: &ContextListItem) -> UserConversationView {
    let short = short_id(item.context_id.as_str());
    let title = item
        .name
        .clone()
        .or_else(|| item.summary.clone())
        .unwrap_or_else(|| format!("Conversation {short}"));
    UserConversationView {
        context_id: item.context_id.clone(),
        context_id_short: short,
        title,
        summary: item.summary.clone(),
        model: item.model.clone(),
        request_count: item.request_count,
        message_count: item.message_count,
        error_count: item.error_count,
        has_errors: item.error_count > 0,
        cost_display: format_cost(item.total_cost_microdollars),
        first_seen_local: item.first_request_at.map(local_stamp),
        last_seen_local: item.last_activity_at.map(local_stamp),
        context_url: context_detail_url(&item.context_id),
    }
}

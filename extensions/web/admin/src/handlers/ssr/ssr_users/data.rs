//! Reads the user detail page makes beyond the account row itself.
//!
//! The roster's own reads moved to `repositories::users::roster`, which answers
//! the whole page in one statement. What is left here is the conversation list
//! the Usage tab shows, which is a cross-table listing rather than a user row.

use sqlx::PgPool;

use crate::repositories;

use super::view;

// Why: the Usage tab shows a person's recent conversations, not their whole
// history — the contexts page, which this tab links to, is where the full
// list with its own paging lives.
const CONVERSATION_LIMIT: i64 = 50;

// Why: no project filter — this is the admin's view of one named account, so
// the project scoping that guards the cross-user contexts list has nothing to
// add and would only hide rows from an admin who asked for them by id.
pub(super) async fn load_conversations(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> Vec<super::super::types::UserConversationView> {
    let filter = repositories::analytics::contexts_list::ContextListFilter {
        user_id: Some(user_id.clone()),
        limit: CONVERSATION_LIMIT,
        ..Default::default()
    };
    let page = repositories::analytics::contexts_list::ContextPage {
        sort: repositories::analytics::contexts_list::ContextSort::default(),
        limit: CONVERSATION_LIMIT,
        offset: 0,
    };
    let (rows, _total) =
        repositories::analytics::contexts_list::list_context_list_paged(pool, &filter, page)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "user detail: conversation list failed"))
            .unwrap_or_default();
    rows.iter().map(view::conversation_view).collect()
}

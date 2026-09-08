//! Loading the conversations list: one page of conversations under the
//! caller's scope, the by-user rollup beside it, and the facets the filter
//! form offers.
//!
//! Every read degrades to an empty result rather than failing the page: a
//! listing that loses its model facet is still worth rendering, and the count
//! above the table is the paged query's own so the two cannot disagree.

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::analytics::conversation_rows::{
    ConversationPage, ConversationRow, ConversationSort, ConversationTotals,
    UserConversationSummary, get_conversation_totals, list_conversation_models,
    list_conversations_paged, list_user_conversation_summaries,
};
use crate::repositories::scope::SubjectScope;

use super::{ContextsPageInputs, PAGE_SIZE, view};

// Why: the by-user tab nests each person's conversations under their row, so
// it reads one wide page newest-first and groups it, rather than one query
// per person.
const USERS_VIEW_ROWS: i64 = 400;
const USER_SUMMARY_LIMIT: i64 = 200;

pub(super) struct ContextsPageData {
    pub(super) conversations: Vec<ConversationRow>,
    pub(super) total_conversations: i64,
    pub(super) user_summaries: Vec<UserConversationSummary>,
    pub(super) totals: ConversationTotals,
    pub(super) models: Vec<String>,
    pub(super) users_for_filter: Vec<crate::types::UserSummary>,
}

pub(super) async fn load_page_data(
    pool: &PgPool,
    inputs: &ContextsPageInputs,
    user_scope: &SubjectScope,
) -> ContextsPageData {
    let filter = &inputs.filter;
    let conversation_page = if inputs.view_is_users {
        ConversationPage {
            sort: ConversationSort::Activity,
            descending: true,
            limit: USERS_VIEW_ROWS,
            offset: 0,
        }
    } else {
        ConversationPage {
            sort: inputs.sort,
            descending: inputs.descending,
            limit: PAGE_SIZE,
            offset: view::page_offset(inputs.page),
        }
    };
    let (list, summaries, totals, models, users) = tokio::join!(
        list_conversations_paged(pool, filter, conversation_page),
        list_user_conversation_summaries(pool, filter, USER_SUMMARY_LIMIT),
        get_conversation_totals(pool, filter),
        list_conversation_models(pool),
        repositories::users::queries::list_users(pool, user_scope),
    );
    let (conversations, total_conversations) = list.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "list_conversations_paged failed");
        (Vec::new(), 0)
    });
    ContextsPageData {
        conversations,
        total_conversations,
        user_summaries: warn_empty(summaries, "user summaries"),
        totals: totals.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_conversation_totals failed");
            ConversationTotals::default()
        }),
        models: warn_empty(models, "models"),
        users_for_filter: warn_empty(users, "users"),
    }
}

fn warn_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.inspect_err(|e| tracing::warn!(error = %e, facet = what, "conversations list: read failed"))
        .unwrap_or_default()
}

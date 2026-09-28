//! Load only the active view; its rows and KPIs share one filtered aggregate.
use super::{ContextsPageInputs, view};
use sqlx::PgPool;
use systemprompt_web_admin_core::repositories::conversations::rows::{
    ConversationPage, ConversationPageMode, ConversationRow, ConversationTotals,
    UserConversationSummary, list_distinct_models, load_conversation_page,
};
use systemprompt_web_admin_core::repositories::scope::SubjectScope;
use systemprompt_web_admin_core::ssr::list_view::DEFAULT_PAGE_SIZE;

pub(super) struct ContextsPageData {
    pub(super) conversations: Vec<ConversationRow>,
    pub(super) total_conversations: i64,
    pub(super) user_summaries: Vec<UserConversationSummary>,
    pub(super) totals: ConversationTotals,
    pub(super) models: Vec<String>,
    pub(super) users_for_filter: Vec<systemprompt_web_admin_core::types::UserSummary>,
}

pub(super) async fn load_page_data(
    pool: &PgPool,
    inputs: &ContextsPageInputs,
    user_scope: &SubjectScope,
) -> Result<ContextsPageData, sqlx::Error> {
    let page = ConversationPage {
        sort: inputs.sort,
        descending: inputs.descending,
        limit: DEFAULT_PAGE_SIZE,
        offset: view::page_offset(inputs.page),
    };
    let mode = if inputs.view_is_users {
        ConversationPageMode::Users
    } else {
        ConversationPageMode::All
    };
    let (result, models, users) = tokio::join!(
        load_conversation_page(pool, &inputs.filter, page, mode),
        list_distinct_models(pool),
        systemprompt_web_admin_core::repositories::principal::listing::list_users(pool, user_scope),
    );
    let result = result?;
    Ok(ContextsPageData {
        total_conversations: result.totals.conversations,
        conversations: result.conversations,
        user_summaries: result.user_summaries,
        totals: result.totals,
        models: warn_empty(models, "models"),
        users_for_filter: warn_empty(users, "users"),
    })
}

fn warn_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &'static str) -> Vec<T> {
    res.inspect_err(|e| tracing::warn!(error = %e, facet = what, "conversations list: read failed"))
        .unwrap_or_default()
}

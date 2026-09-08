//! The by-user rollup of a conversation list: how many conversations each
//! person had under the filter, what they cost, which models they touched, and
//! the latest one so the row can link straight into it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use super::{
    ConversationFilter, ConversationPage, ConversationPageMode, ConversationRow,
    load_conversation_page,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConversationSummary<Id = ContextId> {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub conversation_count: i64,
    pub turn_count: i64,
    pub total_tokens: i64,
    pub side_call_count: i64,
    pub total_cost_microdollars: i64,
    pub last_at: Option<DateTime<Utc>>,
    pub latest: Option<ConversationRow<Id>>,
    pub models: Vec<String>,
}

/// Compatibility wrapper for callers that only render user summaries.
pub async fn list_user_conversation_summaries(
    pool: &PgPool,
    filter: &ConversationFilter,
    limit: i64,
) -> Result<Vec<UserConversationSummary>, sqlx::Error> {
    Ok(load_conversation_page(
        pool,
        filter,
        ConversationPage {
            limit,
            ..ConversationPage::default()
        },
        ConversationPageMode::Users,
    )
    .await?
    .user_summaries)
}

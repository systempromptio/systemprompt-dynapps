//! A single statement shares filtered metrics across the page and its KPIs.

use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use super::{
    ConversationFilter, ConversationPage, ConversationRow, ConversationTotals,
    UserConversationSummary,
};

#[derive(Debug, Default, Deserialize)]
pub struct ConversationPageResult {
    pub conversations: Vec<ConversationRow>,
    pub user_summaries: Vec<UserConversationSummary>,
    pub totals: ConversationTotals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationPageMode {
    All,
    Users,
    Totals,
}

pub async fn load_conversation_page(
    pool: &PgPool,
    filter: &ConversationFilter,
    page: ConversationPage,
    mode: ConversationPageMode,
) -> Result<ConversationPageResult, sqlx::Error> {
    let started = std::time::Instant::now();
    let mut connection = pool.acquire().await?;
    let pool_wait_ms = started.elapsed().as_secs_f64() * 1000.0;
    let query_started = std::time::Instant::now();
    let pattern = filter.free_text_pattern();
    let result = sqlx::query_file!(
        "src/repositories/analytics/conversation_rows/page.sql",
        filter.user_id.as_ref().map(UserId::as_str),
        filter.subject_ids.as_deref(),
        filter.model,
        pattern,
        filter.since,
        filter.until,
        filter.include_side_calls,
        filter.error_only,
        page.sort.as_str(),
        page.descending,
        page.limit,
        page.offset,
        mode == ConversationPageMode::Users,
        mode == ConversationPageMode::Totals,
    )
    .fetch_one(&mut *connection)
    .await?;
    tracing::debug!(
        ?mode,
        pool_wait_ms,
        query_ms = query_started.elapsed().as_secs_f64() * 1000.0,
        "conversation page loaded"
    );
    Ok(result.payload.0)
}

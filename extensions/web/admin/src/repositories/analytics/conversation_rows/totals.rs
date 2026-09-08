//! The KPI strip above a conversation list, computed over the same filter the
//! list pages through; optionally over the preceding window of equal length
//! so each figure can carry a delta.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::ConversationFilter;
use crate::util::time_range::TimeRange;

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct ConversationTotals {
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub side_calls: i64,
    pub tool_calls: i64,
    pub error_conversations: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
    pub side_call_cost_microdollars: i64,
}

pub async fn get_conversation_totals(
    pool: &PgPool,
    filter: &ConversationFilter,
) -> Result<ConversationTotals, sqlx::Error> {
    let pattern = filter.free_text_pattern();
    sqlx::query_as!(
        ConversationTotals,
        r#"
        SELECT
            COUNT(*)::bigint                                          AS "conversations!",
            COUNT(DISTINCT r.user_id)::bigint                         AS "users!",
            COALESCE(SUM(r.turn_count), 0)::bigint                    AS "turns!",
            COALESCE(SUM(r.side_call_count), 0)::bigint               AS "side_calls!",
            COALESCE(SUM(r.tool_call_count), 0)::bigint               AS "tool_calls!",
            COUNT(*) FILTER (WHERE r.error_count > 0)::bigint         AS "error_conversations!",
            COALESCE(SUM(r.total_input_tokens + r.total_output_tokens), 0)::bigint
                                                                      AS "total_tokens!",
            COALESCE(SUM(r.total_cost_microdollars), 0)::bigint       AS "total_cost_microdollars!",
            COALESCE(SUM(r.side_call_cost_microdollars), 0)::bigint   AS "side_call_cost_microdollars!"
        FROM conversation_rollups r
        WHERE ($1::text IS NULL OR r.user_id = $1)
          AND ($2::text[] IS NULL OR r.user_id = ANY($2))
          AND ($3::text IS NULL OR r.model = $3)
          AND ($4::text IS NULL
               OR r.title ILIKE $4
               OR r.display_name ILIKE $4
               OR r.context_id ILIKE $4
               OR r.user_id ILIKE $4)
          AND ($5::timestamptz IS NULL OR r.last_at >= $5)
          AND ($6::timestamptz IS NULL OR r.last_at < $6)
          AND ($7::boolean OR r.turn_count > 0)
          AND (NOT $8::boolean OR r.error_count > 0)
        "#,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.subject_ids.as_deref(),
        filter.model,
        pattern,
        filter.since,
        filter.until,
        filter.include_side_calls,
        filter.error_only,
    )
    .fetch_one(pool)
    .await
}

// Why: the range overrides the filter's own `since`/`until`; the previous
// window is the same span ending where this one starts.
pub async fn get_conversation_totals_windowed(
    pool: &PgPool,
    filter: &ConversationFilter,
    range: TimeRange,
) -> Result<(ConversationTotals, ConversationTotals), sqlx::Error> {
    let span = range.to - range.from;
    let current = ConversationFilter {
        since: Some(range.from),
        until: Some(range.to),
        ..filter.clone()
    };
    let previous = ConversationFilter {
        since: Some(range.from - span),
        until: Some(range.from),
        ..filter.clone()
    };
    let (current, previous) = tokio::join!(
        get_conversation_totals(pool, &current),
        get_conversation_totals(pool, &previous),
    );
    Ok((current?, previous?))
}

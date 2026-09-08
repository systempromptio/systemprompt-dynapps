//! The paged conversation list — one `conversation_rollups` row per
//! conversation under the caller's filter, with the filtered total on every
//! row so the page count needs no second statement.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use super::{ConversationFilter, ConversationPage, ConversationRow};

#[derive(Debug)]
struct PagedRow {
    context_id: ContextId,
    title: String,
    user_id: Option<UserId>,
    display_name: Option<String>,
    session_id: Option<SessionId>,
    client_session_id: Option<String>,
    group_name: Option<String>,
    project_name: Option<String>,
    model: Option<String>,
    turn_count: i64,
    side_call_count: i64,
    side_call_cost_microdollars: i64,
    tool_call_count: i64,
    error_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cost_microdollars: i64,
    first_at: Option<DateTime<Utc>>,
    last_at: Option<DateTime<Utc>>,
    status: Option<String>,
    total_count: i64,
}

impl From<PagedRow> for ConversationRow {
    fn from(r: PagedRow) -> Self {
        Self {
            context_id: r.context_id,
            title: r.title,
            user_id: r.user_id,
            display_name: r.display_name,
            session_id: r.session_id,
            client_session_id: r.client_session_id,
            group_name: r.group_name,
            project_name: r.project_name,
            model: r.model,
            turn_count: r.turn_count,
            side_call_count: r.side_call_count,
            side_call_cost_microdollars: r.side_call_cost_microdollars,
            tool_call_count: r.tool_call_count,
            error_count: r.error_count,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            first_at: r.first_at,
            last_at: r.last_at,
            status: r.status,
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn list_conversations_paged(
    pool: &PgPool,
    filter: &ConversationFilter,
    page: ConversationPage,
) -> Result<(Vec<ConversationRow>, i64), sqlx::Error> {
    let pattern = filter.free_text_pattern();
    let rows = sqlx::query_as!(
        PagedRow,
        r#"
        SELECT
            r.context_id                AS "context_id!: ContextId",
            r.title                     AS "title!",
            r.user_id                   AS "user_id?: UserId",
            r.display_name              AS "display_name?",
            r.session_id                AS "session_id?: SessionId",
            r.client_session_id         AS "client_session_id?",
            r.group_name                AS "group_name?",
            r.project_name              AS "project_name?",
            r.model                     AS "model?",
            r.turn_count                AS "turn_count!",
            r.side_call_count           AS "side_call_count!",
            r.side_call_cost_microdollars AS "side_call_cost_microdollars!",
            r.tool_call_count           AS "tool_call_count!",
            r.error_count               AS "error_count!",
            r.total_input_tokens        AS "total_input_tokens!",
            r.total_output_tokens       AS "total_output_tokens!",
            r.total_cost_microdollars   AS "total_cost_microdollars!",
            r.first_at                  AS "first_at?",
            r.last_at                   AS "last_at?",
            r.status                    AS "status?",
            COUNT(*) OVER ()::bigint    AS "total_count!"
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
        ORDER BY
            CASE WHEN $9 = 'activity' AND     $10::boolean THEN r.last_at END DESC NULLS LAST,
            CASE WHEN $9 = 'activity' AND NOT $10::boolean THEN r.last_at END ASC  NULLS LAST,
            CASE WHEN $9 = 'turns'    AND     $10::boolean THEN r.turn_count END DESC,
            CASE WHEN $9 = 'turns'    AND NOT $10::boolean THEN r.turn_count END ASC,
            CASE WHEN $9 = 'tokens'   AND     $10::boolean
                 THEN r.total_input_tokens + r.total_output_tokens END DESC,
            CASE WHEN $9 = 'tokens'   AND NOT $10::boolean
                 THEN r.total_input_tokens + r.total_output_tokens END ASC,
            CASE WHEN $9 = 'cost'     AND     $10::boolean THEN r.total_cost_microdollars END DESC,
            CASE WHEN $9 = 'cost'     AND NOT $10::boolean THEN r.total_cost_microdollars END ASC,
            r.context_id
        LIMIT $11 OFFSET $12
        "#,
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
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    Ok((rows.into_iter().map(ConversationRow::from).collect(), total))
}

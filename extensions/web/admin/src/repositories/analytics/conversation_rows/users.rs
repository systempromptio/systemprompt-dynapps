//! The by-user rollup of a conversation list: how many conversations each
//! person had under the filter, what they cost, which models they touched, and
//! the latest one so the row can link straight into it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use super::{ConversationFilter, ConversationRow};

#[derive(Debug, Clone, Serialize)]
pub struct UserConversationSummary {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub conversation_count: i64,
    pub turn_count: i64,
    pub side_call_count: i64,
    pub total_cost_microdollars: i64,
    pub last_at: Option<DateTime<Utc>>,
    pub latest: Option<ConversationRow>,
    pub models: Vec<String>,
}

#[derive(Debug)]
struct SummaryRow {
    user_id: UserId,
    display_name: Option<String>,
    conversation_count: i64,
    turn_count: i64,
    side_call_count: i64,
    total_cost_microdollars: i64,
    last_at: Option<DateTime<Utc>>,
    models: Vec<String>,
    l_context_id: Option<ContextId>,
    l_title: Option<String>,
    l_session_id: Option<SessionId>,
    l_client_session_id: Option<String>,
    l_group_name: Option<String>,
    l_project_name: Option<String>,
    l_model: Option<String>,
    l_turn_count: Option<i64>,
    l_side_call_count: Option<i64>,
    l_side_call_cost_microdollars: Option<i64>,
    l_tool_call_count: Option<i64>,
    l_error_count: Option<i64>,
    l_total_input_tokens: Option<i64>,
    l_total_output_tokens: Option<i64>,
    l_total_cost_microdollars: Option<i64>,
    l_first_at: Option<DateTime<Utc>>,
    l_last_at: Option<DateTime<Utc>>,
    l_status: Option<String>,
}

impl From<SummaryRow> for UserConversationSummary {
    fn from(r: SummaryRow) -> Self {
        let latest = r.l_context_id.map(|context_id| ConversationRow {
            context_id,
            title: r.l_title.unwrap_or_default(),
            user_id: Some(r.user_id.clone()),
            display_name: r.display_name.clone(),
            session_id: r.l_session_id,
            client_session_id: r.l_client_session_id,
            group_name: r.l_group_name,
            project_name: r.l_project_name,
            model: r.l_model,
            turn_count: r.l_turn_count.unwrap_or_default(),
            side_call_count: r.l_side_call_count.unwrap_or_default(),
            side_call_cost_microdollars: r.l_side_call_cost_microdollars.unwrap_or_default(),
            tool_call_count: r.l_tool_call_count.unwrap_or_default(),
            error_count: r.l_error_count.unwrap_or_default(),
            total_input_tokens: r.l_total_input_tokens.unwrap_or_default(),
            total_output_tokens: r.l_total_output_tokens.unwrap_or_default(),
            total_cost_microdollars: r.l_total_cost_microdollars.unwrap_or_default(),
            first_at: r.l_first_at,
            last_at: r.l_last_at,
            status: r.l_status,
        });
        Self {
            user_id: r.user_id,
            display_name: r.display_name,
            conversation_count: r.conversation_count,
            turn_count: r.turn_count,
            side_call_count: r.side_call_count,
            total_cost_microdollars: r.total_cost_microdollars,
            last_at: r.last_at,
            latest,
            models: r.models,
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn list_user_conversation_summaries(
    pool: &PgPool,
    filter: &ConversationFilter,
    limit: i64,
) -> Result<Vec<UserConversationSummary>, sqlx::Error> {
    let pattern = filter.free_text_pattern();
    let rows = sqlx::query_as!(
        SummaryRow,
        r#"
        WITH f AS (
            SELECT r.* FROM conversation_rollups r
            WHERE r.user_id IS NOT NULL
              AND ($1::text IS NULL OR r.user_id = $1)
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
        ),
        per_user AS (
            SELECT
                f.user_id,
                MAX(f.display_name)                             AS display_name,
                COUNT(*)::bigint                                AS conversation_count,
                COALESCE(SUM(f.turn_count), 0)::bigint          AS turn_count,
                COALESCE(SUM(f.side_call_count), 0)::bigint     AS side_call_count,
                COALESCE(SUM(f.total_cost_microdollars), 0)::bigint AS total_cost_microdollars,
                MAX(f.last_at)                                  AS last_at
            FROM f
            GROUP BY f.user_id
        ),
        models AS (
            SELECT cr.user_id,
                   ARRAY_AGG(DISTINCT cr.model) FILTER (WHERE cr.model IS NOT NULL) AS models
            FROM conversation_requests cr
            JOIN f ON f.context_id = cr.context_id
            GROUP BY cr.user_id
        ),
        latest AS (
            SELECT DISTINCT ON (f.user_id) f.*
            FROM f
            ORDER BY f.user_id, f.last_at DESC NULLS LAST, f.context_id
        )
        SELECT
            p.user_id                       AS "user_id!: UserId",
            p.display_name                  AS "display_name?",
            p.conversation_count            AS "conversation_count!",
            p.turn_count                    AS "turn_count!",
            p.side_call_count               AS "side_call_count!",
            p.total_cost_microdollars       AS "total_cost_microdollars!",
            p.last_at                       AS "last_at?",
            COALESCE(m.models, ARRAY[]::text[]) AS "models!: Vec<String>",
            l.context_id                    AS "l_context_id?: ContextId",
            l.title                         AS "l_title?",
            l.session_id                    AS "l_session_id?: SessionId",
            l.client_session_id             AS "l_client_session_id?",
            l.group_name                    AS "l_group_name?",
            l.project_name                  AS "l_project_name?",
            l.model                         AS "l_model?",
            l.turn_count                    AS "l_turn_count?",
            l.side_call_count               AS "l_side_call_count?",
            l.side_call_cost_microdollars   AS "l_side_call_cost_microdollars?",
            l.tool_call_count               AS "l_tool_call_count?",
            l.error_count                   AS "l_error_count?",
            l.total_input_tokens            AS "l_total_input_tokens?",
            l.total_output_tokens           AS "l_total_output_tokens?",
            l.total_cost_microdollars       AS "l_total_cost_microdollars?",
            l.first_at                      AS "l_first_at?",
            l.last_at                       AS "l_last_at?",
            l.status                        AS "l_status?"
        FROM per_user p
        LEFT JOIN models m ON m.user_id = p.user_id
        LEFT JOIN latest l ON l.user_id = p.user_id
        ORDER BY p.last_at DESC NULLS LAST, p.user_id
        LIMIT $9
        "#,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.subject_ids.as_deref(),
        filter.model,
        pattern,
        filter.since,
        filter.until,
        filter.include_side_calls,
        filter.error_only,
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(UserConversationSummary::from)
        .collect())
}

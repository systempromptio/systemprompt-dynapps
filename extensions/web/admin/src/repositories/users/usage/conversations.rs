//! The profile pane's conversation rollup: how many conversations a user had
//! in the trailing window, split by model, plus the most recent few.
//!
//! Split out of `usage` because it is the one part of that module answering a
//! different question — not "what did this account spend" but "what did it
//! talk about" — and it is the half with the legacy-sentinel rule that needs
//! stating in one place.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use super::{CONVERSATION_WINDOW_DAYS, ConversationGroup, ConversationSummary, RecentConversation};

// Why: `ai_requests` has no agent column today; the existing analytics surface
// reads agent ids from `plugin_usage_events`, which is keyed differently.
pub async fn get_conversation_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<ConversationSummary, sqlx::Error> {
    let (total_conversations, total_ai_requests) = get_conversation_totals(pool, user_id).await?;
    let by_model = list_conversation_by_model(pool, user_id).await?;
    let recent = list_recent_conversations(pool, user_id).await?;

    Ok(ConversationSummary {
        window_days: CONVERSATION_WINDOW_DAYS,
        total_conversations,
        total_ai_requests,
        by_model,
        by_agent: Vec::new(),
        recent,
    })
}

async fn get_conversation_totals(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<(i64, i64), sqlx::Error> {
    let legacy = ContextId::legacy();
    let totals = sqlx::query!(
        r#"SELECT
            COUNT(DISTINCT context_id) FILTER (WHERE context_id <> $2)::bigint
              AS "total_conversations!",
            COUNT(*)::bigint AS "total_ai_requests!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - make_interval(days => $3)"#,
        user_id.as_str(),
        legacy.as_str(),
        CONVERSATION_WINDOW_DAYS,
    )
    .fetch_one(pool)
    .await?;
    Ok((totals.total_conversations, totals.total_ai_requests))
}

async fn list_conversation_by_model(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ConversationGroup>, sqlx::Error> {
    let legacy = ContextId::legacy();
    Ok(sqlx::query!(
        r#"SELECT
            model AS "model!",
            COUNT(DISTINCT context_id) FILTER (WHERE context_id <> $2)::bigint
              AS "conversations!",
            COUNT(*)::bigint AS "ai_requests!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - make_interval(days => $3)
          GROUP BY model
          ORDER BY COUNT(*) DESC
          LIMIT 5"#,
        user_id.as_str(),
        legacy.as_str(),
        CONVERSATION_WINDOW_DAYS,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| ConversationGroup {
        name: r.model,
        conversations: r.conversations,
        ai_requests: r.ai_requests,
    })
    .collect())
}

// Why: the legacy sentinel groups every context-less request from every
// gateway client into one pseudo-conversation, so it stays excluded here —
// a row linking to it would open a context page belonging to no conversation.
async fn list_recent_conversations(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<RecentConversation>, sqlx::Error> {
    let legacy = ContextId::legacy();
    Ok(sqlx::query!(
        r#"WITH ranked AS (
            SELECT
              context_id,
              MAX(created_at) AS last_activity,
              COUNT(*)::bigint AS ai_requests,
              MAX(model) AS model
            FROM ai_requests
            WHERE user_id = $1
              AND context_id <> $2
              AND created_at >= NOW() - make_interval(days => $3)
            GROUP BY context_id
          )
          SELECT
            ranked.context_id AS "context_id!: ContextId",
            uc.name           AS "context_name?",
            ranked.last_activity AS "last_activity!",
            ranked.ai_requests AS "ai_requests!",
            ranked.model
          FROM ranked
          LEFT JOIN user_contexts uc ON uc.context_id = ranked.context_id
          ORDER BY ranked.last_activity DESC
          LIMIT 5"#,
        user_id.as_str(),
        legacy.as_str(),
        CONVERSATION_WINDOW_DAYS,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| RecentConversation {
        context_id: r.context_id,
        context_name: r.context_name,
        last_activity: r.last_activity,
        ai_requests: r.ai_requests,
        model: r.model,
        agent_name: None,
    })
    .collect())
}

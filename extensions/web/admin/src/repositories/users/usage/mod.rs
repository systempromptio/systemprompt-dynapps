//! Per-user usage aggregations against `ai_requests` for the profile pane.
//!
//! # The canonical token total
//!
//! This module and `analytics::site::kpis` must count tokens identically, and
//! for a long time they did not: this one summed `tokens_used`, the dashboard
//! summed `input_tokens + output_tokens`. Both look reasonable alone, and on
//! live data they differ by two orders of magnitude — `tokens_used` is the
//! provider's own total and counts cache reads, which dominate a Claude Code
//! session. One user's month read 276,421 tokens here and 1,644 there.
//!
//! All four columns are nullable, so a bare `SUM(input_tokens + output_tokens)`
//! additionally collapses to NULL wherever either is missing: a silent zero
//! rather than a visible gap.
//!
//! The agreed definition, used by every query that reports a token count:
//!
//! ```sql
//! COALESCE(tokens_used,
//!     COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
//!   + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))
//! ```
//!
//! The provider's total when it gave one, otherwise the components it did give,
//! cache included — cache tokens are tokens the provider processed and billed,
//! so excluding them would make spend and token count tell different stories
//! about the same request.
//!
//! `sqlx::query!` needs a string literal, so this cannot be interpolated from
//! one constant. It is enforced instead by
//! `tests/integration/admin-core/src/usage_reconciliation.rs`, which asserts
//! this module and the dashboard return the same totals for the same user and
//! window — a test that fails the moment the two drift again.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

mod conversations;

pub use conversations::get_conversation_summary;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct UsageWindow {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub previous_cost_microdollars: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelShare {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub token_share: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationGroup {
    pub name: String,
    pub conversations: i64,
    pub ai_requests: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentConversation {
    pub context_id: ContextId,
    pub context_name: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub ai_requests: i64,
    pub model: Option<String>,
    pub agent_name: Option<String>,
}

// Why: the trailing window every conversation rollup on the profile pane is
// Why: computed over. Carried on the summary so the template can label the
// Why: figures with the window they were actually measured across — the label
// Why: used to read "all time" over a 30-day count.
pub const CONVERSATION_WINDOW_DAYS: i32 = 30;

#[derive(Debug, Clone, Serialize)]
pub struct ConversationSummary {
    pub window_days: i32,
    pub total_conversations: i64,
    pub total_ai_requests: i64,
    pub by_model: Vec<ConversationGroup>,
    pub by_agent: Vec<ConversationGroup>,
    pub recent: Vec<RecentConversation>,
}

impl Default for ConversationSummary {
    fn default() -> Self {
        Self {
            window_days: CONVERSATION_WINDOW_DAYS,
            total_conversations: 0,
            total_ai_requests: 0,
            by_model: Vec::new(),
            by_agent: Vec::new(),
            recent: Vec::new(),
        }
    }
}

/// Lifetime gateway rollup for one user, backing the admin user-detail page.
/// Unwindowed on purpose: an admin opening a user's page is asking what that
/// account has ever done, not what it did this month.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct UserGatewayUsage {
    pub requests: i64,
    pub conversations: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub first_request_at: Option<DateTime<Utc>>,
    pub last_request_at: Option<DateTime<Utc>>,
}

// Why: `window_days` is the trailing window; `previous` covers the equivalent
// prior window so the caller can compute a delta.
pub async fn get_usage_window(
    pool: &PgPool,
    user_id: &UserId,
    window_days: i32,
) -> Result<UsageWindow, sqlx::Error> {
    let curr = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "requests!",
            COALESCE(SUM(COALESCE(tokens_used,
                COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
              + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))), 0)::bigint AS "tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - make_interval(days => $2)"#,
        user_id.as_str(),
        window_days,
    )
    .fetch_one(pool)
    .await?;

    let prev = sqlx::query!(
        r#"SELECT COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
           FROM ai_requests
           WHERE user_id = $1
             AND created_at >= NOW() - make_interval(days => $2 * 2)
             AND created_at <  NOW() - make_interval(days => $2)"#,
        user_id.as_str(),
        window_days,
    )
    .fetch_one(pool)
    .await?;

    Ok(UsageWindow {
        requests: curr.requests,
        tokens: curr.tokens,
        cost_microdollars: curr.cost,
        previous_cost_microdollars: Some(prev.cost),
    })
}

// Why: `window_days: None` is the lifetime view the admin user-detail page
// wants; `Some(n)` is the trailing window the profile pane reports.
pub async fn list_top_models(
    pool: &PgPool,
    user_id: &UserId,
    window_days: Option<i32>,
    limit: i64,
) -> Result<Vec<ModelShare>, sqlx::Error> {
    let total = sqlx::query!(
        r#"SELECT COALESCE(SUM(COALESCE(tokens_used,
                COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
              + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))), 0)::bigint AS "tokens!"
           FROM ai_requests
           WHERE user_id = $1
             AND ($2::int IS NULL
                  OR created_at >= NOW() - make_interval(days => $2))"#,
        user_id.as_str(),
        window_days,
    )
    .fetch_one(pool)
    .await?
    .tokens;

    let rows = sqlx::query!(
        r#"SELECT
            model AS "model!",
            COUNT(*)::bigint AS "requests!",
            COALESCE(SUM(COALESCE(tokens_used,
                COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
              + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))), 0)::bigint AS "tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
          FROM ai_requests
          WHERE user_id = $1
            AND ($2::int IS NULL
                 OR created_at >= NOW() - make_interval(days => $2))
          GROUP BY model
          ORDER BY SUM(COALESCE(tokens_used,
                COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
              + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))) DESC NULLS LAST
          LIMIT $3"#,
        user_id.as_str(),
        window_days,
        limit,
    )
    .fetch_all(pool)
    .await?;

    let total_f = total as f64;
    Ok(rows
        .into_iter()
        .map(|r| ModelShare {
            model: r.model,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
            token_share: if total_f > 0.0 {
                r.tokens as f64 / total_f
            } else {
                0.0
            },
        })
        .collect())
}

// Why: the admin user-detail page needs a lifetime rollup, not the profile
// pane's trailing window — so no interval predicate here at all. The legacy
// sentinel is filtered out of the conversation count only: its requests are
// real traffic and still belong in the request, token, and cost totals.
pub async fn get_ai_request_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<UserGatewayUsage, sqlx::Error> {
    let legacy = ContextId::legacy();
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "requests!",
            COUNT(DISTINCT context_id) FILTER (WHERE context_id <> $2)::bigint
              AS "conversations!",
            COUNT(*) FILTER (WHERE status = 'failed')::bigint AS "failed!",
            COALESCE(SUM(COALESCE(tokens_used,
                COALESCE(input_tokens, 0) + COALESCE(output_tokens, 0)
              + COALESCE(cache_read_tokens, 0) + COALESCE(cache_creation_tokens, 0))), 0)::bigint AS "tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!",
            MIN(created_at) AS "first_request_at?",
            MAX(created_at) AS "last_request_at?"
          FROM ai_requests
          WHERE user_id = $1"#,
        user_id.as_str(),
        legacy.as_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(UserGatewayUsage {
        requests: row.requests,
        conversations: row.conversations,
        failed: row.failed,
        tokens: row.tokens,
        cost_microdollars: row.cost,
        first_request_at: row.first_request_at,
        last_request_at: row.last_request_at,
    })
}

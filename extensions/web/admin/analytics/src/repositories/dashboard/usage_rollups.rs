//! Set-based recompute of `admin_usage_daily_rollups`.
//!
//! Three idempotent upserts — hook-plane counters, session counts, and
//! gateway requests — each recomputing a trailing window of
//! whole days so re-running never double-counts. Dates are UTC to match the
//! hook path's `Utc::now().date_naive()` bucketing. The prompt/tool/error
//! predicates mirror `increment_session_summary`'s classification, so the
//! rollup and the session summaries cannot disagree about what an event is.
//!
//! A fourth pass stamps each row with a group and project. That is exclusive
//! attribution: one container each, so a container's history sums back to the
//! instance total instead of double-counting anyone who belongs to two. The
//! answer is read from `ai_request_scopes` — the scope pinned to each request
//! when it landed — so re-running the rollup after someone moves never rewrites
//! the days they spent elsewhere. A day with hook activity but no gateway
//! request has nothing pinned and takes the person's current primary instead.
//! A person who moved mid-day takes the scope most of that day's requests
//! carry, ties broken by id so the answer is stable across runs.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

pub async fn upsert_daily_rollups_for_window(
    pool: &PgPool,
    days_back: i32,
) -> Result<u64, MarketplaceError> {
    let mut written = 0u64;
    written += rollup_hook_counters(pool, days_back).await?;
    written += rollup_sessions(pool, days_back).await?;
    written += rollup_gateway_requests(pool, days_back).await?;
    written += rollup_attribution(pool, days_back).await?;
    Ok(written)
}

async fn rollup_hook_counters(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups
            (user_id, date, prompts, tool_uses, errors)
        SELECT
            user_id,
            date,
            COALESCE(SUM(event_count) FILTER (WHERE event_type LIKE '%UserPromptSubmit%'), 0),
            COALESCE(SUM(event_count)
                FILTER (WHERE event_type IN ('PostToolUse', 'PostToolUseFailure')), 0),
            COALESCE(SUM(error_count), 0)
        FROM plugin_usage_daily
        WHERE date >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, date
        ON CONFLICT (user_id, date) DO UPDATE SET
            prompts = EXCLUDED.prompts,
            tool_uses = EXCLUDED.tool_uses,
            errors = EXCLUDED.errors,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_sessions(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups (user_id, date, sessions_count)
        SELECT
            user_id,
            (started_at AT TIME ZONE 'UTC')::DATE,
            COUNT(*)::INT
        FROM plugin_session_summaries
        WHERE started_at IS NOT NULL
          AND (started_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, (started_at AT TIME ZONE 'UTC')::DATE
        ON CONFLICT (user_id, date) DO UPDATE SET
            sessions_count = EXCLUDED.sessions_count,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_gateway_requests(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r#"
        INSERT INTO admin_usage_daily_rollups
            (user_id, date, ai_requests_count, input_tokens, output_tokens, cost_microdollars)
        SELECT
            user_id,
            (created_at AT TIME ZONE 'UTC')::DATE,
            COUNT(*)::BIGINT,
            COALESCE(SUM(input_tokens), 0)::BIGINT,
            COALESCE(SUM(output_tokens), 0)::BIGINT,
            COALESCE(SUM(cost_microdollars), 0)::BIGINT
        FROM ai_requests
        WHERE NOT COALESCE(synthetic, FALSE)
          AND (created_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        GROUP BY user_id, (created_at AT TIME ZONE 'UTC')::DATE
        ON CONFLICT (user_id, date) DO UPDATE SET
            ai_requests_count = EXCLUDED.ai_requests_count,
            input_tokens = EXCLUDED.input_tokens,
            output_tokens = EXCLUDED.output_tokens,
            cost_microdollars = EXCLUDED.cost_microdollars,
            updated_at = NOW()
        "#,
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

async fn rollup_attribution(pool: &PgPool, days_back: i32) -> Result<u64, MarketplaceError> {
    Ok(sqlx::query!(
        r"
        WITH stamped AS (
            SELECT DISTINCT ON (t.user_id, t.day) t.user_id, t.day, t.group_id, t.project_id
            FROM (
                SELECT a.user_id,
                       (a.created_at AT TIME ZONE 'UTC')::DATE AS day,
                       s.group_id, s.project_id, COUNT(*) AS requests
                FROM ai_requests a
                JOIN ai_request_scopes s ON s.request_id = a.id
                WHERE NOT COALESCE(a.synthetic, FALSE)
                  AND (a.created_at AT TIME ZONE 'UTC')::DATE >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
                GROUP BY 1, 2, 3, 4
            ) t
            ORDER BY t.user_id, t.day, t.requests DESC, t.group_id, t.project_id
        ), resolved AS (
            SELECT r.user_id, r.date,
                   CASE WHEN st.user_id IS NOT NULL THEN st.group_id ELSE d.primary_group_id END AS group_id,
                   CASE WHEN st.user_id IS NOT NULL THEN st.project_id ELSE d.primary_project_id END AS project_id
            FROM admin_usage_daily_rollups r
            LEFT JOIN stamped st ON st.user_id = r.user_id AND st.day = r.date
            LEFT JOIN user_scope_defaults d ON d.user_id = r.user_id
            WHERE r.date >= (NOW() AT TIME ZONE 'UTC')::DATE - ($1::INT)
        )
        UPDATE admin_usage_daily_rollups r
        SET group_id = x.group_id,
            project_id = x.project_id,
            updated_at = NOW()
        FROM resolved x
        WHERE x.user_id = r.user_id
          AND x.date = r.date
          AND (r.group_id IS DISTINCT FROM x.group_id
               OR r.project_id IS DISTINCT FROM x.project_id)
        ",
        days_back,
    )
    .execute(pool)
    .await?
    .rows_affected())
}

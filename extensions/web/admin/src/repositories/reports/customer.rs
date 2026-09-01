//! A project's month, as its people are allowed to see it.
//!
//! Nothing here selects `cost_microdollars`. What we paid a provider is an
//! internal figure, and a report that carries it is one screenshot away from
//! becoming a pricing discussion. Keeping the column out of the query — rather
//! than out of the template — means the guarantee survives someone adding a
//! field to the view-model later.
//!
//! The scope is a [`ProjectScope`]: every project, or one of them. A user with
//! no project on file folds into `unassigned` rather than dropping out of the
//! totals.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use crate::util::project_scope::ProjectScope;

/// The header figures: who was active and what they used.
#[derive(Debug, Clone, Copy)]
pub struct CustomerMonthSummary {
    // Why: Users that actually made a request in the month.
    pub active_users: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub error_count: i64,
}

impl CustomerMonthSummary {
    #[must_use]
    pub const fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

pub async fn get_customer_month_summary(
    pool: &PgPool,
    scope: &ProjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<CustomerMonthSummary, MarketplaceError> {
    let r = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "requests!",
            COUNT(DISTINCT r.user_id)::BIGINT AS "active_users!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!",
            COUNT(*) FILTER (WHERE r.status NOT IN ('success', 'completed'))::BIGINT
                AS "error_count!"
        FROM ai_requests r
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT IS NULL OR upe.project = $3)
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_one(pool)
    .await?;

    Ok(CustomerMonthSummary {
        active_users: r.active_users,
        requests: r.requests,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        cache_read_tokens: r.cache_read_tokens,
        error_count: r.error_count,
    })
}

/// One user's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerUserUsage {
    pub email: String,
    pub display_name: String,
    pub project: Option<String>,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub distinct_models: i64,
}

// Why: Users with no activity are omitted: their row would be a line of
// zeroes, and on a busy month that is most of the table.
pub async fn list_customer_month_users(
    pool: &PgPool,
    scope: &ProjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerUserUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.email AS "email!",
            COALESCE(NULLIF(u.display_name, ''), u.name) AS "display_name!",
            upe.project AS "project?",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COUNT(DISTINCT r.model)::BIGINT AS "distinct_models!"
        FROM ai_requests r
        JOIN users u ON u.id = r.user_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT IS NULL OR upe.project = $3)
        GROUP BY u.id, u.email, u.display_name, u.name, upe.project
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC, COUNT(*) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerUserUsage {
            email: r.email,
            display_name: r.display_name,
            project: r.project,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            distinct_models: r.distinct_models,
        })
        .collect())
}

/// One project's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerProjectUsage {
    pub project: String,
    pub members: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

pub async fn list_customer_month_projects(
    pool: &PgPool,
    scope: &ProjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerProjectUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(upe.project, 'unassigned') AS "project!",
            COUNT(DISTINCT r.user_id)::BIGINT AS "members!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!"
        FROM ai_requests r
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT IS NULL OR upe.project = $3)
        GROUP BY 1
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerProjectUsage {
            project: r.project,
            members: r.members,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
        })
        .collect())
}

/// One model's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerModelUsage {
    pub provider: String,
    pub model: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
}

pub async fn list_customer_month_models(
    pool: &PgPool,
    scope: &ProjectScope,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerModelUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(r.provider, 'unrouted') AS "provider!",
            COALESCE(r.model, 'unrouted') AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!"
        FROM ai_requests r
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE NOT r.synthetic
          AND r.created_at >= $1 AND r.created_at < $2
          AND ($3::TEXT IS NULL OR upe.project = $3)
        GROUP BY 1, 2
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC
        "#,
        from,
        to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerModelUsage {
            provider: r.provider,
            model: r.model,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
        })
        .collect())
}

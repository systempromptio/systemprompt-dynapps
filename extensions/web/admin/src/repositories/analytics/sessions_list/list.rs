//! Paged session list — one row per session id, with its gateway and hook
//! rollups side by side, plus the primary group and project the person is
//! attributed to.

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

use super::{SessionListFilter, SessionListItem, SessionPage};
use crate::util::time_range::TimeRange;

#[derive(Debug)]
struct SessionRow {
    session_id: SessionId,
    user_id: Option<UserId>,
    display_name: Option<String>,
    group_name: Option<String>,
    project_name: Option<String>,
    model: Option<String>,
    ai_title: Option<String>,
    plugin_id: Option<PluginId>,
    client_source: Option<String>,
    status: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    request_count: i64,
    context_count: i64,
    trace_count: i64,
    tool_uses: i64,
    error_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cost_microdollars: i64,
    has_gateway: bool,
    has_hooks: bool,
    total_count: i64,
}

impl From<SessionRow> for SessionListItem {
    fn from(r: SessionRow) -> Self {
        Self {
            session_id: r.session_id,
            user_id: r.user_id,
            display_name: r.display_name,
            group_name: r.group_name,
            project_name: r.project_name,
            model: r.model,
            ai_title: r.ai_title,
            plugin_id: r.plugin_id,
            client_source: r.client_source,
            status: r.status,
            started_at: r.started_at,
            last_activity_at: r.last_activity_at,
            request_count: r.request_count,
            context_count: r.context_count,
            trace_count: r.trace_count,
            tool_uses: r.tool_uses,
            error_count: r.error_count,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            has_gateway: r.has_gateway,
            has_hooks: r.has_hooks,
        }
    }
}

// Why: The count ignores `page` and covers every row the filter matches, so a
// caller can render "page N of M" without a second round trip.
//
// The sort is a closed `(column, dir)` pair bound as two text parameters and
// selected by a `CASE` in the `ORDER BY`, so the statement stays one
// compile-time-checked literal rather than an interpolated string.
#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn list_sessions_paged(
    pool: &PgPool,
    filter: &SessionListFilter,
    range: TimeRange,
    page: SessionPage,
) -> Result<(Vec<SessionListItem>, i64), sqlx::Error> {
    let sort_col = page.sort.column.as_str();
    let sort_dir = page.sort.dir.as_str();
    let rows = sqlx::query_as!(
        SessionRow,
        r#"
        WITH req AS (
            SELECT
                session_id,
                MAX(user_id)                                       AS user_id,
                COUNT(*)::bigint                                   AS request_count,
                COUNT(DISTINCT context_id)::bigint                 AS context_count,
                COUNT(DISTINCT trace_id)::bigint                   AS trace_count,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint  AS error_count,
                COALESCE(SUM(input_tokens), 0)::bigint             AS total_input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint            AS total_output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint        AS total_cost_microdollars,
                MIN(created_at)                                    AS first_seen,
                MAX(created_at)                                    AS last_seen,
                (ARRAY_AGG(model ORDER BY created_at DESC))[1]     AS model
            FROM ai_requests
            WHERE session_id IS NOT NULL
            GROUP BY session_id
        ),
        joined AS (
            SELECT
                COALESCE(s.session_id, req.session_id)              AS session_id,
                COALESCE(s.user_id, req.user_id)                    AS user_id,
                u.display_name                                      AS display_name,
                g.name                                              AS group_name,
                p.name                                              AS project_name,
                COALESCE(s.model, req.model)                        AS model,
                s.ai_title                                          AS ai_title,
                s.plugin_id                                         AS plugin_id,
                s.client_source                                     AS client_source,
                s.status                                            AS status,
                COALESCE(s.started_at, req.first_seen)              AS started_at,
                GREATEST(
                    COALESCE(req.last_seen, s.ended_at, s.started_at),
                    COALESCE(s.ended_at, s.started_at, req.last_seen)
                )                                                   AS last_activity_at,
                COALESCE(req.request_count, 0)                      AS request_count,
                COALESCE(req.context_count, 0)                      AS context_count,
                COALESCE(req.trace_count, 0)                        AS trace_count,
                COALESCE(s.tool_uses, 0)                            AS tool_uses,
                COALESCE(req.error_count, 0) + COALESCE(s.errors, 0) AS error_count,
                COALESCE(req.total_input_tokens, 0)
                    + COALESCE(s.total_input_tokens, 0)             AS total_input_tokens,
                COALESCE(req.total_output_tokens, 0)
                    + COALESCE(s.total_output_tokens, 0)            AS total_output_tokens,
                COALESCE(req.total_cost_microdollars, 0)            AS total_cost_microdollars,
                (req.session_id IS NOT NULL)                        AS has_gateway,
                (s.session_id IS NOT NULL)                          AS has_hooks
            FROM plugin_session_summaries s
            FULL OUTER JOIN req ON req.session_id = s.session_id
            LEFT JOIN users u ON u.id = COALESCE(s.user_id, req.user_id)
            LEFT JOIN user_scope_defaults d ON d.user_id = u.id
            LEFT JOIN groups g ON g.id = d.primary_group_id
            LEFT JOIN projects p ON p.id = d.primary_project_id
        ),
        filtered AS (
            SELECT j.* FROM joined j
            WHERE j.last_activity_at >= $1
              AND j.last_activity_at < $2
              AND ($3::text IS NULL OR j.user_id = $3)
              AND (NOT $4 OR j.error_count > 0)
              AND ($5::TEXT[] IS NULL OR j.user_id = ANY($5))
        ),
        counted AS (
            SELECT f.*, COUNT(*) OVER ()::bigint AS total_count FROM filtered f
        )
        SELECT
            session_id              AS "session_id!: SessionId",
            user_id                 AS "user_id?: UserId",
            display_name            AS "display_name?",
            group_name              AS "group_name?",
            project_name            AS "project_name?",
            model                   AS "model?",
            ai_title                AS "ai_title?",
            plugin_id               AS "plugin_id?: PluginId",
            client_source           AS "client_source?",
            status                  AS "status?",
            started_at              AS "started_at?",
            last_activity_at        AS "last_activity_at?",
            request_count           AS "request_count!",
            context_count           AS "context_count!",
            trace_count             AS "trace_count!",
            tool_uses               AS "tool_uses!",
            error_count             AS "error_count!",
            total_input_tokens      AS "total_input_tokens!",
            total_output_tokens     AS "total_output_tokens!",
            total_cost_microdollars AS "total_cost_microdollars!",
            has_gateway             AS "has_gateway!",
            has_hooks               AS "has_hooks!",
            total_count             AS "total_count!"
        FROM counted
        ORDER BY
            CASE WHEN $6 = 'started_at' AND $7 = 'desc' THEN started_at END DESC NULLS LAST,
            CASE WHEN $6 = 'started_at' AND $7 = 'asc'  THEN started_at END ASC  NULLS LAST,
            CASE WHEN $6 = 'duration' AND $7 = 'desc'
                 THEN EXTRACT(EPOCH FROM (last_activity_at - started_at)) END DESC NULLS LAST,
            CASE WHEN $6 = 'duration' AND $7 = 'asc'
                 THEN EXTRACT(EPOCH FROM (last_activity_at - started_at)) END ASC NULLS LAST,
            CASE WHEN $6 = 'requests' AND $7 = 'desc' THEN request_count END DESC NULLS LAST,
            CASE WHEN $6 = 'requests' AND $7 = 'asc'  THEN request_count END ASC  NULLS LAST,
            CASE WHEN $6 = 'tokens' AND $7 = 'desc'
                 THEN total_input_tokens + total_output_tokens END DESC NULLS LAST,
            CASE WHEN $6 = 'tokens' AND $7 = 'asc'
                 THEN total_input_tokens + total_output_tokens END ASC NULLS LAST,
            CASE WHEN $6 = 'cost' AND $7 = 'desc' THEN total_cost_microdollars END DESC NULLS LAST,
            CASE WHEN $6 = 'cost' AND $7 = 'asc'  THEN total_cost_microdollars END ASC  NULLS LAST,
            session_id
        LIMIT $8 OFFSET $9
        "#,
        range.from,
        range.to,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.error_only,
        filter.subject_ids.as_deref(),
        sort_col,
        sort_dir,
        page.limit,
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    Ok((rows.into_iter().map(SessionListItem::from).collect(), total))
}

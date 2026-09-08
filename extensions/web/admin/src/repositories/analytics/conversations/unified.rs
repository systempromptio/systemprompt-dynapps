//! One conversation list over both places a conversation can be recorded.
//!
//! A Claude Code session is captured by the Stop hook into
//! `session_transcripts`; a `/v1/messages` call through the gateway is
//! captured into `ai_requests`, grouped by `context_id`. A user who only ever
//! talks to the gateway has zero transcript rows, which is why
//! `/admin/history` used to show them nothing at all while their traffic was
//! plainly visible to an admin on the contexts list.
//!
//! Both halves are read in one statement so paging and the total count are
//! computed over the union rather than per source. Ranked search runs against
//! the transcript `search_tsv` index as before; the gateway half has no
//! full-text index, so it matches by `ILIKE` over the context name, the
//! opening prompt, the id, and the model, and always ranks below a scored
//! transcript hit.
//!
//! A session that has both a Stop-hook transcript and gateway rows appears
//! twice, once per source. That is deliberate: the two records are captured by
//! different mechanisms and hold different fields, and collapsing them would
//! mean silently choosing one of them to show.
//!
//! The gateway half reads `conversation_requests`, so a turn is a request
//! whose `effective_kind` is `turn`; probes and utility calls are counted as
//! side calls and a conversation made of side calls alone is listed only when
//! the caller asks for them.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

/// Where a conversation was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HistorySource {
    Transcript,
    Gateway,
}

impl HistorySource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Transcript => "Claude Code",
            Self::Gateway => "Gateway",
        }
    }

    fn parse(raw: &str) -> Self {
        if raw == "gateway" {
            Self::Gateway
        } else {
            Self::Transcript
        }
    }
}

/// One conversation in the unified history list. Exactly one of `session_id`
/// and `context_id` is set, according to `source`.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryItem {
    pub source: HistorySource,
    pub session_id: Option<SessionId>,
    pub context_id: Option<ContextId>,
    pub user_id: UserId,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_at: DateTime<Utc>,
    pub turns: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub cost_microdollars: i64,
    pub side_call_count: i64,
    pub rank: Option<f32>,
    pub snippet: Option<String>,
}

#[expect(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "body is one irreducible compile-time-checked query! SQL literal; every argument is one bound parameter"
)]
pub async fn list_history_items(
    pool: &PgPool,
    scope_user_ids: Option<&[String]>,
    search: Option<&str>,
    include_side_calls: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<HistoryItem>, i64), sqlx::Error> {
    let query = search.map(str::trim).filter(|q| !q.is_empty());
    let pattern = query.map(|q| format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%")));
    let legacy = ContextId::legacy();

    let rows = sqlx::query!(
        r#"WITH classified AS MATERIALIZED (
            SELECT cr.* FROM conversation_requests cr
            WHERE cr.context_id = ANY(ARRAY(
                SELECT DISTINCT context_id FROM ai_requests
                WHERE ($2::text[] IS NULL OR user_id = ANY($2)) AND context_id <> $5
            ))
        ), latest AS (
            SELECT DISTINCT ON (st.session_id)
                st.id, st.session_id, st.user_id, st.model, st.captured_at,
                COALESCE(st.entries_counted, 0) AS entries_counted,
                COALESCE(st.total_input_tokens, 0) AS total_input_tokens,
                COALESCE(st.total_output_tokens, 0) AS total_output_tokens,
                CASE WHEN $1::text IS NULL THEN NULL
                     ELSE ts_rank(st.search_tsv, websearch_to_tsquery('english', $1))
                END AS rank
            FROM session_transcripts st
            WHERE ($2::text[] IS NULL OR st.user_id = ANY($2))
              AND ($1::text IS NULL
                   OR st.search_tsv @@ websearch_to_tsquery('english', $1))
            ORDER BY st.session_id, st.captured_at DESC
        ),
        transcripts AS (
            SELECT
                'transcript'::text                       AS source,
                l.session_id                             AS session_id,
                NULL::text                               AS context_id,
                l.user_id                                AS user_id,
                pss.ai_title                             AS title,
                NULL::text                               AS preview,
                l.model                                  AS model,
                COALESCE(pss.started_at, l.captured_at)  AS started_at,
                l.captured_at                            AS last_at,
                l.entries_counted::bigint                AS turns,
                l.total_input_tokens::bigint             AS tokens_in,
                l.total_output_tokens::bigint            AS tokens_out,
                0::bigint                                AS cost_microdollars,
                0::bigint                                AS side_calls,
                l.rank                                   AS rank,
                l.id AS transcript_id
            FROM latest l
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = l.session_id
        ),
        gw AS (
            SELECT
                context_id,
                MAX(client_session_id)                      AS client_session_id,
                COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint  AS turns,
                COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_calls,
                COALESCE(SUM(input_tokens), 0)::bigint      AS tokens_in,
                COALESCE(SUM(output_tokens), 0)::bigint     AS tokens_out,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
                MIN(created_at)                             AS started_at,
                MAX(created_at)                             AS last_at
            FROM classified
            WHERE context_id <> $5
              AND ($2::text[] IS NULL OR user_id = ANY($2))
            GROUP BY context_id
        ),
        gw_latest AS (
            SELECT DISTINCT ON (context_id) context_id, user_id, model
            FROM classified
            WHERE context_id <> $5
            ORDER BY context_id, created_at DESC
        ),
        gateway AS (
            SELECT
                'gateway'::text        AS source,
                NULL::text             AS session_id,
                g.context_id           AS context_id,
                l.user_id              AS user_id,
                COALESCE(pss.ai_title, NULLIF(uc.name, 'Gateway conversation')) AS title,
                CASE WHEN $6::text IS NOT NULL THEN conversation_opening_prompt(g.context_id, 200) END AS preview,
                l.model                AS model,
                g.started_at           AS started_at,
                g.last_at              AS last_at,
                g.turns                AS turns,
                g.tokens_in            AS tokens_in,
                g.tokens_out           AS tokens_out,
                g.cost_microdollars    AS cost_microdollars,
                g.side_calls           AS side_calls,
                NULL::real             AS rank,
                NULL::text             AS transcript_id
            FROM gw g
            JOIN gw_latest l ON l.context_id = g.context_id
            LEFT JOIN user_contexts uc ON uc.context_id = g.context_id
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = g.client_session_id
            WHERE ($7::bool OR g.turns > 0)
              AND ($6::text IS NULL
                   OR pss.ai_title ILIKE $6
                   OR uc.name ILIKE $6
                   OR conversation_opening_prompt(g.context_id, 200) ILIKE $6
                   OR g.context_id ILIKE $6
                   OR l.model ILIKE $6)
        ),
        unified AS (
            SELECT * FROM transcripts
            UNION ALL
            SELECT * FROM gateway
        )
        , counted AS MATERIALIZED (
            SELECT *, COUNT(*) OVER ()::bigint AS total_count FROM unified
        ), page AS MATERIALIZED (
            SELECT * FROM counted
            ORDER BY rank DESC NULLS LAST, last_at DESC, source, COALESCE(session_id, context_id)
            LIMIT $3 OFFSET $4
        )
        SELECT
            source                          AS "source!",
            session_id                      AS "session_id?: SessionId",
            context_id                      AS "context_id?: ContextId",
            COALESCE(user_id, '')           AS "user_id!: UserId",
            title                           AS "title?",
            COALESCE(preview, CASE WHEN source = 'gateway'
                THEN conversation_opening_prompt(context_id, 200) END) AS "preview?",
            model                           AS "model?",
            started_at                      AS "started_at?",
            last_at                         AS "last_at!",
            turns                           AS "turns!",
            tokens_in                       AS "tokens_in!",
            tokens_out                      AS "tokens_out!",
            cost_microdollars               AS "cost_microdollars!",
            side_calls                      AS "side_calls!",
            rank                            AS "rank?",
            CASE WHEN $1::text IS NOT NULL AND transcript_id IS NOT NULL THEN
                (SELECT ts_headline('english', left(st.transcript::text, 262144),
                    websearch_to_tsquery('english', $1),
                    'StartSel=[, StopSel=], MaxWords=24, MinWords=8, MaxFragments=2')
                 FROM session_transcripts st WHERE st.id = transcript_id)
            END AS "snippet?",
            total_count AS "total_count!"
        FROM page
        ORDER BY rank DESC NULLS LAST, last_at DESC, source, COALESCE(session_id, context_id)"#,
        query,
        scope_user_ids,
        limit,
        offset,
        legacy.as_str(),
        pattern,
        include_side_calls,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let items = rows
        .into_iter()
        .map(|r| HistoryItem {
            source: HistorySource::parse(&r.source),
            session_id: r.session_id,
            context_id: r.context_id,
            user_id: r.user_id,
            title: r.title,
            preview: r.preview,
            model: r.model,
            started_at: r.started_at,
            last_at: r.last_at,
            turns: r.turns,
            total_input_tokens: r.tokens_in,
            total_output_tokens: r.tokens_out,
            cost_microdollars: r.cost_microdollars,
            side_call_count: r.side_calls,
            rank: r.rank,
            snippet: r.snippet,
        })
        .collect();
    Ok((items, total))
}

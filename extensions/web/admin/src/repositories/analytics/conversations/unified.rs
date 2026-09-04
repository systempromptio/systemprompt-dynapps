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
    pub rank: Option<f32>,
    pub snippet: Option<String>,
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query! SQL literal"
)]
pub async fn list_history_items(
    pool: &PgPool,
    scope_user_ids: Option<&[String]>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<HistoryItem>, i64), sqlx::Error> {
    let query = search.map(str::trim).filter(|q| !q.is_empty());
    let pattern = query.map(|q| format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%")));
    let legacy = ContextId::legacy();

    let rows = sqlx::query!(
        r#"WITH latest AS (
            SELECT DISTINCT ON (st.session_id)
                st.session_id, st.user_id, st.model, st.transcript, st.captured_at,
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
                l.rank                                   AS rank,
                CASE WHEN $1::text IS NULL THEN NULL
                     ELSE ts_headline('english', left(l.transcript::text, 262144),
                                      websearch_to_tsquery('english', $1),
                                      'StartSel=[, StopSel=], MaxWords=24, MinWords=8, MaxFragments=2')
                END                                      AS snippet
            FROM latest l
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = l.session_id
        ),
        gw AS (
            SELECT
                context_id,
                MAX(user_id)                                AS user_id,
                COUNT(*)::bigint                            AS turns,
                COALESCE(SUM(input_tokens), 0)::bigint      AS tokens_in,
                COALESCE(SUM(output_tokens), 0)::bigint     AS tokens_out,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
                MIN(created_at)                             AS started_at,
                MAX(created_at)                             AS last_at,
                (ARRAY_AGG(model ORDER BY created_at DESC))[1] AS model
            FROM ai_requests
            WHERE context_id <> $5
              AND ($2::text[] IS NULL OR user_id = ANY($2))
            GROUP BY context_id
        ),
        -- Opening user turn per context: the gateway stores a turn as a debug
        -- transcript, so the marker framing and the assistant half are cut
        -- before truncating, exactly as the contexts list does.
        gw_preview AS (
            SELECT DISTINCT ON (r.context_id)
                r.context_id,
                LEFT(
                    NULLIF(
                        BTRIM(regexp_replace(
                            regexp_replace(
                                split_part(m.content, '=== ASSISTANT ANSWER ===', 1),
                                '=== USER PROMPT ===', '', 'g'),
                            '\s+', ' ', 'g')),
                        ''),
                    200) AS preview
            FROM ai_request_messages m
            JOIN ai_requests r ON r.id = m.request_id
            WHERE r.context_id <> $5 AND m.role = 'user'
            ORDER BY r.context_id, r.created_at ASC, m.sequence_number ASC
        ),
        gateway AS (
            SELECT
                'gateway'::text        AS source,
                NULL::text             AS session_id,
                g.context_id           AS context_id,
                g.user_id              AS user_id,
                uc.name                AS title,
                p.preview              AS preview,
                g.model                AS model,
                g.started_at           AS started_at,
                g.last_at              AS last_at,
                g.turns                AS turns,
                g.tokens_in            AS tokens_in,
                g.tokens_out           AS tokens_out,
                g.cost_microdollars    AS cost_microdollars,
                NULL::real             AS rank,
                NULL::text             AS snippet
            FROM gw g
            LEFT JOIN user_contexts uc ON uc.context_id = g.context_id
            LEFT JOIN gw_preview p ON p.context_id = g.context_id
            WHERE $6::text IS NULL
               OR uc.name ILIKE $6
               OR p.preview ILIKE $6
               OR g.context_id ILIKE $6
               OR g.model ILIKE $6
        ),
        unified AS (
            SELECT * FROM transcripts
            UNION ALL
            SELECT * FROM gateway
        )
        SELECT
            source                          AS "source!",
            session_id                      AS "session_id?: SessionId",
            context_id                      AS "context_id?: ContextId",
            COALESCE(user_id, '')           AS "user_id!: UserId",
            title                           AS "title?",
            preview                         AS "preview?",
            model                           AS "model?",
            started_at                      AS "started_at?",
            last_at                         AS "last_at!",
            turns                           AS "turns!",
            tokens_in                       AS "tokens_in!",
            tokens_out                      AS "tokens_out!",
            cost_microdollars               AS "cost_microdollars!",
            rank                            AS "rank?",
            snippet                         AS "snippet?",
            (COUNT(*) OVER ())::bigint      AS "total_count!"
        FROM unified
        ORDER BY rank DESC NULLS LAST, last_at DESC
        LIMIT $3 OFFSET $4"#,
        query,
        scope_user_ids,
        limit,
        offset,
        legacy.as_str(),
        pattern,
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
            rank: r.rank,
            snippet: r.snippet,
        })
        .collect();
    Ok((items, total))
}
